# EZ-CD

*Pronunciation: Ee-zee See-Dee*

## Instructions

Send file for install:

```shell
ez-cd-cli -f FILE_PATH.deb -d TARGET_HOSTNAME --connect tcp/homepi:7447


ez-cd-cli -f docker_out/hub-system_0.6.22_arm64.deb -d homepi --connect tcp/homepi:7447
```

## Warning

!DO NOT USE THIS PROJECT!

This project poses a massive security risk if you ever allow anyone unauthenticated to connect to the service.
It's basically allowing anyone to execute commands with root privileges.
Can only be used on a local network with no risk or unauthorized connections.
