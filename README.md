# Command Saver

Run and save commands under some name and use them later.  
Useful to save curl requests and execute them later with names.

## Installation

```
cargo install --git https://github.com/var77/cmd-saver
```

## Usage

Run `saver h` to show help  
The command below will execute `curl` and save the command under name `curl1`

```
saver s curl1 curl https://example.com
```

Now when we run `saver l` we will get this output:

```
1) curl1
```

To view the command we can use `saver g curl1` which will print

```
curl https://example.com
```

To run the command again type `saver r curl1`

To delete saved command run `saver d curl1`
