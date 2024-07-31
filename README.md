# rusty-cron-scheduler
Lightweight and performant library to use cron formatted strings to trigger function pointers.

1. [About](#about)
2. [Features](#features)
3. [Installation](#installation)
4. [Usage](#usage)
5. [Contributing](#contributing)

## About
rusty-scheduler is a simple library made to receive function pointers and cron strings and take care of when to execute them with some configurability to better adapt it to most situations, it's a very lightweight implementation and thus is limited to currently triggering simple functions without parameters, I'm currently investigating and prototyping the possibility of adding functions with parameters.

The main scheduler thread runs in a separate thread that starts when startup() is called for the scheduler, you may still use add_task and remove_task after starting the scheduler since a Mutex takes cares of any synchronization issue. 

Another important thing, if the execution of the task doesn't end before it's next execution is due it will skip that execution, there's currently no parallel triggering, this will be configurable in a later release.

And the execution of the tasks is done by a separate thread on the background, so yes, every task you add will end up generating a thread when it executes, but never more than 1 (for now), that's something to keep in mind.

## Features
Currently allows for either a 5 or 6 tokens cron formated string depending on if you need the precision to go up to minutes or seconds respectively.
It also allows all normal tokens to be used in the string, those are:

- *: Defines all possible values
- x/y: Defines values starting at x and repeating every y, for example: 0/5, every 5 minutes starting at 0
- x-y: Values starting at x until y, for example: 5-15, every minute between 5 and 15
- x,y,z: Specific values defined, for example: 1,5,50, at minutes 1, 5 and 50

You can also mix and match those in some ways, checkout https://crontab.guru/ for an amazing explanation on what does your cron do.

You may also configure 2 parameters:

- scheduler_wait_millis: Sets the amount of time in milliseconds that the scheduler sleeps after checking the tasks and execute whatever needed to execute. The lower this number the more precise the scheduler is BUT the more CPU it will consume.
- execution_threshold_millis: This variables is defined both at the task level and the scheduler level, scheduler level is the default one but if the task has it specified that one is prioritized. This variable is used to determine a threhold in milliseconds where the task can execute before the exact specified time, for example, if in the current loop there's still 50 milliseconds to go until the time is reached then it'd have to wait at least "scheduler_wait_millis" to execute, which can cause a big delay depending on the configuration, thus, if the execution_threshold_millis is set to a 100ms it may execute the task at most 100ms before the expected time.

The default values are 1000 for the scheduler wait and 250 for the threshold, but I encorage you to test different configurations to better adapt the scheduler to your needs.

## Installation
Should be as easy as **cargo add rusty-scheduler**

## Usage
To start a new scheduler use Scheduler::new(<config>), either create some SchedulerConfigOptions or send None (defualt config will be used)
Then use **add_task** to send a task to the scheduler, with the parameters: **cron**, string with the cron you want the function to follow, it accepts either 5 or 6 cron tokens (https://crontab.guru is a great website to check your crons), **function_to_exec**, a pointer to a function without any parameters, you can create a pointer like: "let my_ptr = MyClass::MyFunc" and finall y an optional **execution_threshold_millis**, does the same as the shceduler parameter but only for this method.

**add_task** will return a Uuid that you can use to call **remove_task** with, which will delete the task from the scheduler.

Finally call **startup()** on the scheduler to start the scheduler thread, which will do the heavy lifting of generating the threds that will execute your functions

You may still add or remove tasks after startup and the same scheduler thread will keep them tracked

## Contributing
If you feel like something is missing, could be improved or needs to be changed let me know on a ticket or just start a pull request and I'll try to have a look at it asap.