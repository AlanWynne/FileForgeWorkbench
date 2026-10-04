# JCL SYSIN SYSPRINT SYSERR

For a FileForgeWorkbench-style batch environment, I would recommend following common IBM conventions rather than strict IBM requirements.

There is no universal IBM mapping of stdin/stdout/stderr because the DD names are application-defined, but over the years some conventions became extremely common.

Unix/Linux	Common Mainframe Equivalent	Usagestdin	SYSIN	Control statements, input stream data
stdout	SYSPRINT	Program messages, normal report output
stderr	SYSOUT or SYSERR	Error and diagnostic output
logs	SYSLOG	System/operator messages
report output	SYSOUT	Printed reports
spool output	JES SYSOUT class	Batch output destined for spool

My recommendation for FFWB would be:

stdin   -> SYSIN
stdout  -> SYSPRINT
stderr  -> SYSERR

Example:

```jcl
//STEP1    EXEC PGM=MYPROG
//SYSIN    DD *
INPUT DATA
MORE DATA
/*
//SYSPRINT DD SYSOUT=*
//SYSERR   DD SYSOUT=*
```

The program would see:

SYSIN     = reader
SYSPRINT  = standard output writer
SYSERR    = standard error writer

Why SYSERR?

Although not a system-reserved DDNAME, many IBM utilities and vendor products use:

```jcl
//SYSERR DD ...
```

for error output.

This gives a cleaner separation than sending everything to SYSPRINT.

Looking at IBM Utilities
IEFBR14

No input/output DD convention.

Only DD allocation statements matter.

```jcl
//STEP1 EXEC PGM=IEFBR14
//FILE1 DD DSN=...
```

IDCAMS
```jcl
//SYSIN    DD *
 DELETE MY.FILE
/*
//SYSPRINT DD SYSOUT=*
```

SYSIN contains commands.

SYSPRINT contains messages.

DFSORT
```jclS
//SORTIN   DD ...
//SORTOUT  DD ...
//SYSIN    DD *
 SORT FIELDS=...
/*
//SYSPRINT DD SYSOUT=*
```

IEBGENER
```jcl
//SYSUT1   DD ...
//SYSUT2   DD ...
//SYSPRINT DD SYSOUT=*
```

Again SYSPRINT is generally the diagnostics/report stream.

Alternative Mapping

If you want something closer to modern operating systems:

stdin     -> SYSIN
stdout    -> SYSOUT
stderr    -> SYSERR
log       -> SYSPRINT

but this feels less "mainframe-like".

Most experienced z/OS developers mentally associate:

SYSIN      = input
SYSPRINT   = output/messages

so mapping stdout to SYSPRINT tends to feel natural.

My Recommendation for FFWB

Implement these predefined DD names:

SYSIN      Standard Input
SYSPRINT   Standard Output
SYSERR     Standard Error
SYSOUT     Generic Output Dataset
SYSLOG     System Log
SYSTERM    Terminal/Interactive Output

Internally:

stdin   -> SYSIN
stdout  -> SYSPRINT
stderr  -> SYSERR

and allow DD statements to override them:

```jcl
//SYSPRINT DD DSN=JOB.REPORT
//SYSERR   DD DSN=JOB.ERRORS
```

That gives you a very natural JCL execution model while still feeling familiar to Linux developers. It also aligns nicely with your idea of emulating JES, SDSF, IDCAMS, and IEFBR14 within FileForgeWorkbench.