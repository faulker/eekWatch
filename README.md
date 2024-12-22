# eekWatch
A application for checking system resources and on failures alerting.

I created this tool because I couldn't find a easy to use monitoring tool for my personal server that would have the ability to monitor desk usage.

# Config File

You will need to setup the config file by defining your SMTP server and the credentials needed to connect to it, also define logging settings.

Example base config:
```
{
  "debug": false,
  "rules": {
    "location": "rules"
  },
  "alerts": {
    "email": {
      "smtp": "<smtp server>",
      "user": "<smtp username>",
      "password": "<smtp password>",
      "from_address": "<alert email's from address>"
    },
    "logging": {
      "location": "<directory to store logs in>",
      "file": "<log file name>",
      "rotation": {
        "when": "size",
        "limit": "100"
      }
    }
  }
}
```



