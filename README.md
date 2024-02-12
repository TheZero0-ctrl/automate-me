## Automate me
Try to automate everything I have been doing manually thus far. And learn rust at the same time

## Current Automation
### `give-me-article` Command
This command give random article from my reading list on notion based on their priority 

### `generate-stand-up` Command
This command pull the tasks from my task manager page on notion and generate a stand up

#### Options
- -s, --slack : Flag for sending stand up to slack
- -t, --timelog : Flag for updating time log on google sheet
- -i, --in-office : data to fill out on In Office header in google sheet [default: WFH]
- -w, --hours : data to fill out on Hours header in google sheet [default: 8]

### `add-tasks` Command
This command add tasks to my task manager page on notion

#### Options
- -t, --task : task to add
- -s, --status : status of the task, Possible values: "to do", "in progress", "done" [default: done]
- -p, --project : project of the task


more automation to comes...
