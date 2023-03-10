## Dirshell

This small application allows to save commonly used shell commands on folder basis and then quickly recall it with convenient dropdown menu.

![](https://raw.githubusercontent.com/bogvak/dirshell/master/img/demoopt.gif)

Commands may be added to list prefixing command with `dirshell` command, ex

```shell
>> dirshell terragrunt apply --terragrunt-log-level debug --terragrunt-debug
```

or directly with any code editor by editing `.comhistory` file that stored in appropriate folder - **dirshell** keeping commands in that file.

### Environment variables file

if there is files with `.env` extension in current folder - app automatically load environment variables to child process.

If there multiple `.env` files - you will need to select from dropdown menu - which file it's needed to use.

If you do not need to load environment variables - use `dirshell --` command.

### Modify command before run

If you need to modify some common command before running - use `dirshell *` command.

That could be useful if you need, for example, add some rarely used parameter or key to some common used command

### Platforms supported

Currently app has prebuild binary for two platforms - Windows X86_64 ([link](https://github.com/bogvak/dirshell/releases/latest/download/dirshell.exe)) and Linux GNU X86_64 ([link](https://github.com/bogvak/dirshell/releases/latest/download/dirshell)).

More platform may be added by building from source with appropriate build target by your own or if you interested in supporting of some particular platform - don't be shy to contact me directly ([support@bogv.online](mailto:support@bogv.online?subject=Dirshell)), I will do that that for you.

### Tips and tricks

##### Put binary to folder from where you may execute it everywhere

For Windows it could be any folder that was added to system environment variable PATH

For Linux it's recommended to use folder `/usr/local/bin`. Do not forget to assign executable mode for downloaded binary (*chmod +x*).

##### You may rename dirshell and use it under different name

For example you may want to use shorter name but just mind to not override some other existing system command or utility.

**You may filter commands dropdown list**

If your list of folder specific commands is too long but you still want to access some commands quicker - you may start typing in prompt after you run **dirshell** - dropdown will start filtering automatically.