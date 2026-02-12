# Frame Sequences

How frames flow through Prior's kernel. Each diagram is a
[Mermaid sequence diagram](https://mermaid.js.org/syntax/sequenceDiagram.html)
rendered natively by GitHub.

---

## 1. Connection Lifecycle

A gateway client connects, joins a room, sends a message, then disconnects.
Door validates sessions and room membership at every step.

```mermaid
sequenceDiagram
    participant GW as Gateway
    participant Door
    participant Kernel
    participant Room

    GW->>Door: door:connect {from, room?}
    Door->>GW: item {session} + done

    GW->>Door: door:join {session, room}
    Door->>GW: done

    GW->>Door: door:message {session, room, content}
    Note over Door: validate session + membership
    Door->>Kernel: room:message {room, from, content}
    Kernel->>Room: room:message
    Room-->>Kernel: item* + done
    Kernel-->>Door: item* + done
    Door-->>GW: item* + done

    GW->>Door: door:part {session, room}
    Door->>GW: done

    GW->>Door: door:disconnect {session}
    Door->>GW: done
```

---

## 2. Kernel Routing

The kernel's single inbound channel receives every frame. Requests route to
subsystems by syscall prefix. Responses route back to callers by `parent_id`.

```mermaid
sequenceDiagram
    participant Caller
    participant Kernel
    participant VFS as VFS Subsystem
    participant EMS as EMS Subsystem

    Note over Kernel: register("vfs") → VFS pipe<br/>register("ems") → EMS pipe

    Caller->>Kernel: submit(vfs:read) [id=A]
    Note over Kernel: pending[A] = [Caller]
    Kernel->>VFS: vfs:read [id=A]
    VFS-->>Kernel: item [parent_id=A]
    Note over Kernel: pending[A] → Caller
    Kernel-->>Caller: item [parent_id=A]
    VFS-->>Kernel: done [parent_id=A]
    Note over Kernel: pending[A] → Caller, cleanup
    Kernel-->>Caller: done [parent_id=A]

    Caller->>Kernel: submit(ems:select) [id=B]
    Note over Kernel: pending[B] = [Caller]
    Kernel->>EMS: ems:select [id=B]
    EMS-->>Kernel: item [parent_id=B]
    Kernel-->>Caller: item [parent_id=B]
    EMS-->>Kernel: done [parent_id=B]
    Kernel-->>Caller: done [parent_id=B]
```

---

## 3. Subsystem-to-Subsystem Call

Room acts as both server (handling `room:message`) and client (calling
`llm:chat`). Its pipe bridges responses back through the kernel.

```mermaid
sequenceDiagram
    participant Door
    participant Kernel
    participant Room
    participant LLM

    Door->>Kernel: room:message [id=A]
    Kernel->>Room: room:message [id=A]

    Note over Room: build context from history
    Room->>Kernel: llm:chat [id=B]
    Note over Kernel: pending[B] = [Room pipe]
    Kernel->>LLM: llm:chat [id=B]
    LLM-->>Kernel: item {stop_reason, content} [parent_id=B]
    Kernel-->>Room: item [parent_id=B]
    LLM-->>Kernel: done [parent_id=B]
    Kernel-->>Room: done [parent_id=B]

    Note over Room: append reply to history
    Room-->>Kernel: item {from, content} [parent_id=A]
    Kernel-->>Door: item [parent_id=A]
    Room-->>Kernel: done [parent_id=A]
    Kernel-->>Door: done [parent_id=A]
```

---

## 4. LLM Tool Loop

When the LLM returns `stop_reason: "tool_use"`, Room dispatches tool calls
through the kernel and feeds results back to the LLM. Repeats up to 20 rounds.

```mermaid
sequenceDiagram
    participant Room
    participant Kernel
    participant LLM
    participant VFS as Target Subsystem

    loop up to 20 rounds
        Room->>Kernel: llm:chat {messages, tools}
        Kernel->>LLM: llm:chat
        LLM-->>Room: item {stop_reason: "tool_use", content: [tool_use blocks]}

        Note over Room: emit door:thought, door:tool broadcasts

        loop for each tool_use block
            Room->>Kernel: {syscall} (e.g. vfs:read)
            Kernel->>VFS: vfs:read
            VFS-->>Room: item* + done
        end

        Note over Room: append tool_results to context
    end

    Note over Room: final round returns stop_reason: "end_turn"
    Room->>Kernel: llm:chat {messages}
    Kernel->>LLM: llm:chat
    LLM-->>Room: item {stop_reason: "end_turn", content: [text]}
    Note over Room: emit door:chat broadcast
```

---

## 5. Broadcast (door:chat / door:thought / door:tool)

During the actor loop, Room emits fire-and-forget broadcasts via
`Caller::spam()`. These flow through the kernel to Door, which fans them
out to all gateway connections in the target room.

```mermaid
sequenceDiagram
    participant Room
    participant Kernel
    participant Door as Door Subsystem
    participant GW1 as Gateway A
    participant GW2 as Gateway B

    Note over Room: actor produces thinking block
    Room->>Kernel: door:thought {room, from, content}
    Kernel->>Door: door:thought
    Note over Door: DoorRegistry.broadcast(room, frame)
    Door->>GW1: door:thought
    Door->>GW2: door:thought

    Note over Room: actor calls a tool
    Room->>Kernel: door:tool {room, from, syscall}
    Kernel->>Door: door:tool
    Door->>GW1: door:tool
    Door->>GW2: door:tool

    Note over Room: actor produces final reply
    Room->>Kernel: door:chat {room, from, content}
    Kernel->>Door: door:chat
    Door->>GW1: door:chat
    Door->>GW2: door:chat
```

---

## 6. Tick Heartbeat

The kernel sends a tick to every registered subsystem once per second.
Ticks bypass persistence and pending-route tracking.

```mermaid
sequenceDiagram
    participant Kernel
    participant Door
    participant Room
    participant VFS
    participant EMS

    loop every 1 second
        Note over Kernel: broadcast_tick()
        Kernel->>Door: door:tick
        Kernel->>Room: room:tick
        Kernel->>VFS: vfs:tick
        Kernel->>EMS: ems:tick
        Door-->>Kernel: done
        Room-->>Kernel: done
        VFS-->>Kernel: done
        EMS-->>Kernel: done
    end
```

---

## 7. Pipe Internals

How `PipeEnd` and `Caller` manage call/response correlation through the
lazy dispatcher.

```mermaid
sequenceDiagram
    participant Sub as Subsystem (PipeEnd)
    participant Disp as Dispatcher Task
    participant Kernel as Kernel Inbound

    Note over Sub: first call() → ensure_dispatcher()
    Sub->>Sub: spawn dispatcher task

    Sub->>Kernel: request [id=X]
    Note over Sub: pending[X] = CallStream

    Kernel-->>Disp: item [parent_id=X]
    Note over Disp: pending[X] exists → route to CallStream
    Disp-->>Sub: item via CallStream

    Kernel-->>Disp: done [parent_id=X]
    Note over Disp: terminal → cleanup pending[X]
    Disp-->>Sub: done via CallStream

    Kernel-->>Disp: room:tick (no parent_id match)
    Note over Disp: no match → default channel
    Disp-->>Sub: tick via PipeEnd::recv()
```

---

## 8. Gateway Passthrough

Non-door syscalls from gateways are forwarded directly through the kernel
after an allowlist check. Door relays the response stream without inspection.

```mermaid
sequenceDiagram
    participant GW as Gateway
    participant Door
    participant Kernel
    participant VFS

    GW->>Door: vfs:read {path}
    Note over Door: allow_gateway_passthrough("vfs") → true
    Door->>Kernel: vfs:read (via Caller::call)
    Kernel->>VFS: vfs:read
    VFS-->>Kernel: item {content}
    Kernel-->>Door: item
    Door-->>GW: item (relay_stream)
    VFS-->>Kernel: done
    Kernel-->>Door: done
    Door-->>GW: done
```

---

## 9. Error Routing

When a request targets an unknown subsystem, the kernel generates an error
response and routes it back through the normal pending mechanism.

```mermaid
sequenceDiagram
    participant Caller
    participant Kernel

    Caller->>Kernel: submit(bogus:op) [id=C]
    Note over Kernel: pending[C] = [Caller]
    Note over Kernel: routes.get("bogus") → None
    Kernel->>Kernel: error "unknown subsystem: bogus" [parent_id=C]
    Note over Kernel: route_response → pending[C]
    Kernel-->>Caller: error [parent_id=C]
    Note over Kernel: terminal → cleanup pending[C]
```
