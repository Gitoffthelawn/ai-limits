# Provider CLI Virtual Terminal Sessions

## Virtual Terminal and CLI Sessions

The diagram below describes the runtime session for interactive CLIs that require a pseudoterminal.

```mermaid
stateDiagram-v2
    [*] --> Limit_request

    Limit_request --> Provider_selected
    Provider_selected --> Method_selected
    Method_selected --> Session_check

    Session_check --> Open_session: Session exists
    Session_check --> New_session: No session

    Open_session --> Data_request
    New_session --> Data_request

    Data_request --> Data_received
    Data_received --> Normalization
    Normalization --> Limits_shown_to_user

    Limits_shown_to_user --> [*]
```

---

## Runtime Shutdown

A virtual terminal lives only for the duration of the active application runtime. When the runtime terminates, the application must synchronously shut down all open virtual terminals and associated provider sessions.

This rule is for resource control: the application must not create terminals uncontrollably and leave them running after the user exits or the process stops.
