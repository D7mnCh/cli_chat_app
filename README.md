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
- use mpsc (channels) for events (enums), notifiction, one time used (between threads), (can't modify data with it)
- use `Arc<Mutex<...>>` when the shared data is updated continuously (between threads)
- server job is judging, client can only react (i don't quite understand that)
- don't use sleep on networking?
- they must be only one reader on boht client and server side or it will be race condtion on the message between threads
- good structured code make adding features why easier(i implement multiple readers on both client and server -.-, and that leads to code mess like a reader take packets of another reader, the same parsing implemented over and over again)
- logging on every thing i do make debbuging less painful
- you need code review 
- i don't really know when to make a blank line
- if you lock the mutex twice, you will get a deadlock (blocked thread)
- String or &String will coerce into &str 
- use iterators when you want to change items of a collection (computation), else use for loops to execute other than modifying the items (side effect)
- iteraters in rust are lazy, lazy means you need to consume the iterator in order to execute what inside the closuser
- iterators are consumed when next method is called, if next called, it will (consume) that value and become None


# TODO
- introduce the "why you did such a thing" for the thing that you learnt
- search on what make project good or profissional
    - good git commits
        - one logic change per commit
        - new feature = new branch
    - good README.md
        - should put most important stuff (nees samples for that)
    - readble code
        - good comments, the why is more powerfull than the what
        - good architecture disgn
- i should organize those notes cuz some of them are not this project related
- i think i can make both issues and features as (issues) in github rather than a note in README.md
## On server side
- (Access is denied. (os error 5)) i get this error on windows when i try to connect
- os error 10054 i get this error on windows when i crush the program
to server after quit, i need all clients to quit in order to connect on other terminal session
## on client side
 - no more features, organize your project, and try understand the ratatui library, ratatui examples is your friends
### Ui
#### issues
#### Features to add
 - when run server let user input ip address
 - waiting room ui to check if server running
 - jump me on the last message when there many messages
 - i need logging in ratatui context, my project will not scale well if i not did that
 - make logs on popout window (see ratatui examples)
 - set length limit of the client name
## Not Ui:
#### Issues
 - (Access is denied. (os error 6)) i get this error on windows when i try to connect???
 - os error 10054 i get this error on windows when i crush the program
 - when i suspend server, does my program just broke
 - sometimes, the reader doesn't read from stream well
 #### Features
 - retry connection

## big moves
- switch to using async (i'll make it on other branch after i finsih this project, or
it could be before that for learning)
- bind server to wifi, and let client on the same wifi connect to that server,
    need search on how to do that (safely, for now ?)
- use JSON with serde or make your own JSON parsing

# Concepts
- Serialization: converting data into bytes/string, deserializing (the opposite), it will constract the data from bytes/strings
