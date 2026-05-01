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
- if you wanna use continue or break keywords in other function rather tehn the current one, you can do other method by letting that function that don't have the keywords containe the logic and return an enum of dicisions to let know the caller (loop function) what she should do! 
- user input should ui struct handle it
- don't make multiple sources of truth, only one
- use mpsc (channels) for events (enums), notifiction, one time used
- use `Arc<Mutex<...>>` when the shared data is updated continuously
- server job is judging, client can only react (i don't quite understand that)
- don't use sleep on networking?
- they must be only one reader on boht client and server side or it will be race condtion on the message between threads
