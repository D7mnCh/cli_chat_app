# cli_chat_app
# What i learnt
- if you need multiple tasks at the same time, start using threads
- if you want a lifetime of a thread to be in a certain duration, use scope threads
- all spawned threads that are within the `thread::scope`, if one of them is blocking, the next one will not get executed
- need to be '\n' in order to write out to other stream end point
- you can asolate stream and stdin by make them in different threads,(don't make any on the main thread?)
- if client disconnected, it send 0 data as signal (the connection is dead (EOF))
- use `panic!` when something non expected happen
- don't pass all struct on an other struct field, just pass it's fields
- pub keyword found it usefull, if i didn't found it beside method definition, it means i only use that method on associated struct
