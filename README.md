# cli_chat_app
- cli_chat_app (as it name said) a chat app that runs on the terminal, built with rust lang and ratatui (a Tui library)
- the app networking implemented using TcpStream and TcpListener from rust standard library
- implemented very basic parsing for both client/server messages
- it have been tested on both linux and windows (not tested on macos)
# Usage
- install rust first
```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```
- clone this rep
> [!WARNING]
> if you get "No such file or directory", you working direcotry must be on where `Cargo.toml` is
## to run the server
```bash
cargo run --bin --release server
```
## to run client 
```bash
cargo run --bin --release client
```
> [!WARNING]
>  for now, clients must be on the same Wifi in able to connect, so i didn't make server runs on the cloud fro global connections
# Screenshots
- server side
![Alt text](/screenshots/running-server.png)
- client side
![Alt text](/screenshots/running-client.png)

# what i learn from building this project
## Netowrking/concurrency Concepts
- Serialization: converting data into bytes/string, deserializing (the opposite), it will constract the data from bytes/strings
- the writer to stream must have "\n", it tells the reader where does this message ends
- if client disconnected, it send 0 data as signal (the connection is dead (EOF))
- if you tried to shutdown client, on windows (no linux), it will senda an error "os error 10054"
- server job is judging, client can only react, so if client want something, he needs to ask the server first to check if it is valid or not
- they must be only one reader on both client and server side or it will be a race condition on the messages between reader threads
- i used to have mulitple readers, and use sleep thread because of that, to fix that i just switch to one reader and improving my parsing ex. having an item per \n, break the item into 4 parts (for all kind of items)
- use `Arc<Mutex<...>>` when the shared data is updated continuously (between threads)
- use mpsc (channels) on events (enums), notifiction, one time used (can't modify data with it)

## Programming in general
- good structured code make adding features why easier(i implement multiple readers on both client and server -.-)
- you need code review, to check if it scaling well or not
- if you lock the mutex twice, you will get a deadlock (blocked thread)
- if the caller function want break, countinue keywords, but can't and the calle function have them, make the logic on the caller function, and let return an enum of dicisinos, and then use match on that enum on the calle function
- you must not have multiple sources of truth, have only one
- logging on every thing i do make debbuging less painful (i have on that part)
- make a blank line on stuff that seems related
- if you have multiple nested code, and inside that code you have sort of a big logic, then put that logic into a function to increase readabily
- if the resource lives long enough as a struct, and that struct needs it, just makes it as a field
- Rendering must only have (how to draw something), but the logic on when to draw it make it on app struct
- if you found it hard to impl a feature that my project can break if didn't impl it, just downgrade it and impl an easy version of it (ex. set input length limit on input area, cuz i don't know how to impl wrapping on it)

# TODO
## On server side
- (Access is denied. (os error 5)) i get this error on windows when i try to connect
- redesigning network.rs
## On client side
### Ui
#### Features to add
 - use writeln!, so when serialization no need to append "\n" at the end of the returned string
 - when run server let user input ip address
 - waiting room ui to check if server is running
 - i need logging in ratatui context, if i use stdout it broke the ui
 - make logs on popout window (see ratatui examples)
### Not Ui:
#### Features to add
 - use writeln!, so when serialization no need to append "\n" at the end of the returned string
#### Issues
 - redesigning network.rs
 - (Access is denied. (os error 5)) i get this error on windows when i try to connect, and disappear if i restart the server
 - when i suspend server, does my program just broke
 - sometimes, the reader doesn't read from stream well (need to modify unwrap_or_else() i made with Arc<T>)
 #### Features
 - retry connection

## big moves
- introduce async 
- bind server to the cloud, so other clients from differnt Wifi can connect to the same server
    - https://developers.cloudflare.com/cloudflare-one/access-controls/applications/non-http/cloudflared-authentication/arbitrary-tcp/
- use JSON with serde or make your own JSON parsing
- impl encryption on messages
