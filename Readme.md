## Dirshell

This small application allow to save common used shell command history on folder basis and then quickly recall it with convenient dropdown menu.

![demoopt](C:\Private\20-RUST\dirshell\img\demoopt.gif)

Commands to history may be added prefixing command with `dirshell` command, ex

```shell
>> dirshell terragrunt apply --terragrunt-log-level debug --terragrunt-debug
```

or directly with any code editor by editing `.comhistory` file that stored in appropriate folder - **dirshell** keeping commands in that file.

### Platforms supported

Currently app has prebuild binary for two platforms - Windows X86_64 and Linux GNU X86_64.

More platform may be added by building from source with appropriate build target by your own or if you interested in supporting of some particular platform - don't be shy to contact me directly ([support@bogv.online](mailto: support@bogv.online?subject=Dirshell)), I will do that that for you.

### Tips and tricks

##### Put binary to folder from where you may execute it everywhere

For Windows it could be any folder that was added to system environment variable PATH

For Linux it's recommended to use folder `/usr/local/bin`. Do not forget to assign executable mode for downloaded binary (*chmod +x*).