# Task Schedular plugin - FileForgeWorkbench.

I want to build a small lightweight Task(job) scheduler similar to TWS, as a plugin to FFWB. It should have the following capabilities: 

Create a database to store the Schedule metadata.
Currently TWS has the concept of a current plan.  Once a day (Late afternoon) the execution schedule (Current Plan) for the next day is Created,  the Current plan can only be changed via adhoc requests. 
TWS does nto have a long term plan as such only a plan for the next 24 hours. Once a day the next 24 hours schedule is built from the current scheduling database.

Define a background program to schedule jobs.
Define Calandar class.
Define a workstation. A workstation is a locations/machine where tasks execute! Initialy we will support only Local Work Station's (Same machine) later we will include remote workstations.
Define an application. A list of tasks.
Define a task that must execute, it could be of different types, Bash Shell, Windows command shell, powershell script, jcl etc.
Define a Resource name: just a name space the can be marked for how many items may hold it.  0 = unavailble, 1 = single use,  2 = two jobs can hold the resource at a time etc...
Define Triggers:  A trigger is an event to start an application.

Attributes of an Calandar Class.

	Calandar Class Name.
	Calandar Class ID.
	Calandar Class Descritpion.
	Calandar Class Country Code (ISO Couytry Code)
	Calander Parent Unique id (Default the master Calander)
	Days Excluded Class (Days of the Week 1-7)
		Day Shift -1 day before, 0 no shift, +1 Day after Applies if the excluded day falls on an already excluded day. 
	Days Excluded List  (list of Days - Like Public  holidays)
		Day Shift -1 day before, 0 no shift, +1 Day after Applies if the excluded day falls on an already excluded day. 

Notes about the calandar classes: 

	A Calandear class inherits from it's parent class.
	The other attributes will then further limit the clander.
	So for instance if we wanted to build a calander for Business days in South Africa, We would first build a Calander build where a list of days for public holidays are excluded.
	Could be called something like ZA-X-Public-Holidays-Sundays. This will include all days from the master calander, where public Holidays and Sunday's are excluded.
	Due to the Day Sift rule if a public holiday falls on a Sunday the public holiday shifts to the Monday.
	Then we could Create a Calander Called ZA-Business-Days and have as its parent Calander the ZA-X-Public-Holidays and exclude
	We would have to build a Workspace to allow us to edit the attributes of Calendars.
	A Calander Generator - A dispaly very much like the current display on the POM of FFWB, but it should show six months into the future, Active days will be green text, Inactive days Blue Text.
	Perhaps this can be retro fitted into the POM. where the calendar shown in the POM is provided according to the Task Schedular calendar.
	We might want to retro fit the calandar class into the FFWB core then the Calandar display on the POM Can be configured according to a particular Calander.
	What about jobs that must run many times a day?  We need to build a tools to cater for something like this.  What granuality should we allow,  up to microseconds, or is up to every minute,  What is reasonable. this should probably not be part of the Calander Class Plugin.

Attributes of an Workstation:

	Workstation Unique ID.
	Workstation Name.
	Workstation Availbility (0 => Unavailble, 1 => 1 Availalbe, n => n Available). 
	Workstation Use Count ( Number of tasks using the workstation).
	Location the url where the workstation will execute. (default to local Host).
	The Types of executions (Should a workstation support only one kind of execution or should it support different ones , python, powershell, or should this be a configuration of the task.)
	When starting a task on a workstation could this possibly be done via na API? Even if the task is to run in a shell script?
	Perhaps we need to create a workstation Agent program that can run on a workstation. perhaps it can be configured as a task to be scheduled. So workstation availbility can be managed by the schedular as well. Would require some sort of bootstrapping?
		We would need to build Workstation agents for each environment (Windows, UNIX/Linux) Mainframe) 
		The Workstation Agent provides an API to allow other tasks to be started as per the schedule in an environment. Tasks are started by the scheduler Async.
		The Workstation Agent Monitors the Tasks it started, the output and error logs, and then on completion feeds back to the schedular updating it on the tasks status.
		The Workstation Agent Should keep the schedular informed as to the status of the exectuing task and it's own healthTask should update the Schedulars Workstation Availability indicator and keep it up to date, if it ceases to notify it is assumed to be dead, what is a reasonable period for these updates?
		This would require an API also on the schedular side to recieve the response of a tasks success or failure.
	A Work Station attributes should probaly include connection parameters and logon credentails etc.  What is the best practive for storing this? 

Creating updating Workstation Attributes:

	A workspace to set workstation attributes. Workstation Use Count can only be set by the workstation api agent.
	So once a workstation is up and running it will have to provide feedback to the schedular about it's unique id and it's status, and increment the Workstation Availbility count.
	If a workstation is shut down it should provide feed back to the scheduler that it is being shut down, reduce the number of instances available.
	A Workstation Agent can be shut down on a server without the schduling being aware, So the implications is that each workstion instance Should be feeding back to the schedular on a periodic timetable it's status.
	If a workstation ceases to provide feedback to the schedular, it is assumed the workstation is down. should provide an escalation message so that it can be restarted.
	Should we provide a way to start workstation agents from the schedular? 
	It is feasable for a server to run many instances of different scheduled tasks at a time. Should we have one workstion agent per server that can support many instances of that workstation or should we have a workstation agent per instance allowed of the workstion on a server.
	i.e. An Agent that supports many workstation instances, or a workstation agent per instance?
	Workstation Availability can be set by the workstation Agent? 
	This would also 
	Since FFWB is not intended to be the Job Schedular, we would require some form of Workstation Scheduling agent running either locally or remotly where the scheduling happens.
	FFWB would just be enquiring on the Scheduling data base what the status is of the Current plan,  What jobs were scheduled to run what threre status is and hopw far along in the schduling plan we have prpgressed.

Attributes of an Application:

	Application ID.
	Application Name.
	Application Description.
	Applications will be scheduled to run Periodically or Adhoc.
		Periodic jobs must have a calander class selected.
		Applications Predesessors. Could be many, could be other applications or tasks within other applications, Could be time drive, Could be event driven. 
	Optioanlly - 
		Enforced task start time 
		Earliest start time 
		Latest Start time - Escalation Notifications if time not met.
		Expected duration
		Latest End Time - Escalation Notifications if time not met.

Attributes of a Task: 

	Task Application ID.
	Task Name. 
	Task ID.
	Task Target Workstation..
	Task Description.
	Task Run book on failure.
	Task Documentation.
	Exceution script Type (Windows cmd, powershell, powershell7, python? etc...)
	Execution script (Shell script, JCL, A script supported by the Workstation)
		Task Predesessors.  Could be many,  could be tasks within the same application or could be tasks from other applications. 
		if task from other applications are listed as predecessors, but the other application is not loaded for the same scheduling plan, then the tasks are assumed completed.
	Optioanlly - 
		Enforced task start time 
		Earliest start time 
		Latest Start time - Escalation Notifications if time not met.
		Expected duration
		Latest End Time - Escalation Notifications if time not met.
		Task Resources. A List of resources that must be availble for the task to execute.
			obviously one of those resources would be teh workstation.
			other resources are just name spaces that can be logged out for use and a task can only be released for execution once it has grabbed all the resources it requires.

Attributes of a Resource: 

	Resource ID.
	Resource Name.
	Resource Calander.
	Resource Max Availability. ( A number telling us how many items can share this resource ). 0 = unavailble, 1 = single use,  n = n jobs can hold the resource at a time
	Resource Availability ( A number telling us how many items currently share this resource ). 0 = Not in use, 1 = single using,  n = n jobs are using.
		Resource Availability GE Resource Max Availability then jobs must wait for the resource to become availble.
		Resource Availability must be manged by the Task Schedular. it only schedules a task once it has incremmented the Resource Availability for each resource it requires.

Trigger Attributes: 

	A Trigger is pulled by a process or Event that happens, We will have a program to run that is called with a trigger id. This is pulling the Trigger!
	Trigger ID.
	Trigger Name.
	Trigger Descritpion.
	Trigger applications. Could be many.
	
	When a trigger is pulled,  all the applications linked to the Trigger are loaded and started as adhoc applications to the current Scheduling plan.
	