## File Forge Workbench.

	cargo fmt -- --check
	cargo fmt 
	cargo clippy --workspace -- -D warnings
	cargo build  --workspace --release 2>&1 | Select-String 'error|warning' 
	cargo test  2>&1 | Select-String '(?i)error|warning' 
	.\target\debug\ffwb.ex

### simple push

    .\Merge-Repair-MarkdownAndConvert.ps1 -SourceDir . -MasterFile Master.md -OutputFile Master.docx -ReferenceDoc .\reference.docx -TableOfContents -NumberSections

	Copy markdown files form source to Destination
	pandoc master.md -o master.docx
		
    C:\tools\scripts\Copy-Markdown.ps1 -Source "C:\workspace\VSC\FileForgeWorkbench" -Destination "C:\Users\A142367\OneDrive - Standard Bank\01_Active_Projects\FileForgeWorkBench" 
		
	Push project to Github. 	
	.\push-to-github.ps1 -Message "Phase AY: ..." -Tag "phase-ay"
	.\push-to-github.ps1 -Message "Phase AN (partial): ..." -Tag "phase-an-part1"

### New Requirements


#### Record-Oriented Resource Model.

	Break the follwing into task so i can see progress: 
	Examine the following and the the requiremtns of the FFWB and ensure that these requirements
	are also included in the FFWB.
	
	Attached is a ears requirement for a VFS system that can emulate different file systems. Does it emmulate the following: Mainframe files do not use CRLF or LF or CR as record terminators, each file has an attrubute FB or VB,  if the file is FB each record is set to a certain length and the next record follwos immedietly,  if the file is VB,  teh first 4 bytes of a record contain an binary integer specifying the records length and the record follws from there at the end of that length, the next binary integer 

	Short answer: No, not currently.

	I have reviewed the attached requirements.md. The VFS specification defines a generic abstraction layer, URI scheme, provider registry, provider traits, file operations, metadata, search, and watching, but it does not explicitly model mainframe record-oriented files such as FB and VB datasets.

	The closest reference is that the URI supports provider-specific query parameters, with an example such as ?recfm=fb, and that provider-specific metadata may be stored in a metadata map. However, no requirements define the semantics of FB or VB records, record access, RDWs, LRECL, BLKSIZE, ASA control characters, RECFM, or variable-length record handling.

	What your current VFS can represent

	The current specification effectively treats content as:

	Resource
		-> read() -> Vec<u8>
		-> read_stream() -> Byte Stream


	which is a traditional Unix/Windows stream-of-bytes model.

	That works well for:

	Windows files
	Linux files
	MacOS files
	ZIP entries
	Cloud objects
	FTP files


	but not fully for z/OS-style datasets where the primary abstraction is:

	Dataset
		-> Record 1
		-> Record 2
		-> Record 3


	not

	Byte Stream

	What is missing?

	For mainframe compatibility you would normally need support for:

	Record Format (RECFM)

	Examples:

		F
		FB
		V
		VB
		U
		FBA
		VBA

	For your example:

	FB (Fixed Blocked)
	RECFM=FB
	LRECL=80


	Data is physically stored as:

	80 bytes
	80 bytes
	80 bytes
	80 bytes
	...


	No CRLF.

	No LF.

	No record terminator.

	Record boundaries are determined by:

	offset = record_number * LRECL

	VB (Variable Blocked)

	Data is stored as:

	RDW
	Record Data

	RDW
	Record Data

	RDW
	Record Data


	where the RDW is:

	4 bytes


	The first two bytes contain the record length.

	Example:

	00 54 00 00
	<80-byte record>

	00 24 00 00
	<32-byte record>


	Again:

	No LF
	No CR
	No CRLF


	The next RDW determines the start of the next record.

	Logical Record Length (LRECL)

	Need a file attribute:

	LRECL = 80


	or

	LRECL = 32756


	This is fundamental to FB/VB files.

	Currently not defined.

	Block Size (BLKSIZE)

	Need support for:

	BLKSIZE=27920


	or similar.

	Currently not defined.

	Dataset Organisation

	Need attributes such as:

	PS
	PO
	PO-E
	GDG
	VSAM


	Currently not defined in the VFS itself.

	Record-Oriented Access

	Today the VFS defines:

	read() -> Vec<u8>
	read_stream() -> AsyncRead


	You would likely also need:

	read_records()
	write_records()
	append_record()


	For example:

	async fn read_records(
		&self,
		path: &str
	) -> Vec<Record>;

	Architectural concern

	For FileForgeWorkbench this is a very important design decision.

	There are two possible approaches.

	Option 1 - Stream-Based VFS

	Treat every resource as bytes.

	Dataset providers translate records into:

	Record 1 + LF
	Record 2 + LF
	Record 3 + LF


	when opened.

	Advantages:

	Simple
	Compatible with editors

	Disadvantages:

	Loses exact z/OS fidelity
	Rewriting VB files becomes complicated
	
	Option 2 - Record-Aware VFS

	Keep records as first-class objects.

	Example:

	enum ContentModel {
		ByteStream,
		FixedRecords,
		VariableRecords,
	}


	with metadata:

	struct DatasetAttributes {
		recfm: Recfm,
		lrecl: u16,
		blksize: u16,
	}


	Advantages:

	Accurate z/OS emulation
	Supports JES, SDSF, datasets, ASA reports
	Supports true FB/VB editing

	Disadvantages:

	More complexity
	My recommendation

	For FileForgeWorkbench, especially given your defined goals around:

	Dataset Catalog
	Dataset Allocator
	JES emulator
	Mainframe workstation experience
	FileForge structured files

	I would strongly recommend extending the VFS specification with a Record-Oriented Resource Model.

	At a minimum I would add VfsMetadata fields such as:

	content_model
	record_format
	logical_record_length
	block_size
	dataset_organisation
	code_page

	and requirements such as:

	THE VFS SHALL support resources whose native representation is
	record-oriented rather than stream-oriented.

	THE VFS SHALL preserve record boundaries for providers that expose
	record-oriented datasets.

	THE dataset-catalog provider SHALL support FB and VB datasets.

	THE VFS SHALL expose RECFM, LRECL and BLKSIZE as metadata.

	THE VFS SHALL support sequential record iteration without requiring
	materialisation as a newline-delimited text stream.

	Based solely on the contents of requirements.md, those capabilities are not presently specified, 
	so your current VFS requirements would not guarantee correct emulation of FB and VB dataset behaviour.

	Yes, if you extend the VFS the way we discussed, then FFWB could accurately open and process a binary-downloaded mainframe dataset, but there are a few conditions.

	Looking at your current VFS specification, the URI scheme already allows provider-specific options such as:

	vfs://catalog/HLQ.FILE?encoding=ebcdic&recfm=fb


	which suggests that encoding and record format were already on your mind when the specification was written.

	Also, your overall FileForgeWorkbench specification explicitly includes:

	FileForge Integration
	EBCDIC support
	COMP-3 support
	Dataset Catalog emulation
	Dataset Allocator
	Mainframe dataset catalog concepts

	These indicate that alternative encodings and mainframe file structures are expected somewhere in the architecture.

	What happens with a binary download?

	Suppose you FTP a dataset in binary mode.

	FB Example

	Dataset:

	RECFM=FB
	LRECL=80


	Downloaded file:

	80 bytes
	80 bytes
	80 bytes
	80 bytes
	...


	No CRLF.

	No LF.

	No separators.

	To Windows Notepad the file appears as:

	One giant line


	because there are no newline characters.

	If FFWB knows:

	RECFM=FB
	LRECL=80
	CODEPAGE=IBM-1047


	then it can reconstruct:

	Record 1 = bytes 0-79
	Record 2 = bytes 80-159
	Record 3 = bytes 160-239


	and display:

	line 1
	line 2
	line 3


	even though no CRLF exists in the source file.

	VB Example

	Suppose:

	RECFM=VB


	The downloaded file contains:

	RDW
	Record Data
	RDW
	Record Data
	RDW
	Record Data


	FFWB would:

	Read RDW
	Determine record length
	Extract record
	Read next RDW
	Continue

	Result:

	Record 1
	Record 2
	Record 3


	displayed correctly in the editor.

	The physical file remains unchanged.

	The critical requirement

	The binary file alone is not enough.

	You need metadata describing it.

	For example:

	Dataset Name
	RECFM
	LRECL
	BLKSIZE
	CODEPAGE
	DSORG


	Without that metadata:

	PROJECT.COBOL.SOURCE


	could be interpreted as:

	FB 80
	FB 133
	VB
	VBA


	and produce completely different record boundaries.

	How a catalog solves this

	Your Dataset Catalog is actually the ideal place to store this.

	Instead of:

	catalog entry
		-> file path


	store:

	catalog entry
		-> file path
		-> RECFM
		-> LRECL
		-> BLKSIZE
		-> CODEPAGE
		-> DSORG


	Example:

	{
	  "dsn": "HLQ.COBOL.SOURCE",
	  "path": "c:/catalog/hlq.cobol.source.dat",
	  "recfm": "FB",
	  "lrecl": 80,
	  "blksize": 27920,
	  "codepage": "IBM-1047"
	}


	Then FFWB has everything required to reconstruct the original dataset.

	Code pages

	You mentioned alternative code pages.

	The VFS document itself only explicitly gives an example query parameter:

	?encoding=ebcdic


	It does not define a code-page model or a list of supported code pages.

	So from the specification I can safely say:

	The design anticipates alternate encodings, but the exact code-page handling requirements are not formally specified in the VFS requirements document.

	For a true mainframe workstation I would recommend adding:

	pub enum Encoding {
		Utf8,
		Utf16,
		Ascii,
		Ebcdic1047,
		Ebcdic037,
		Ebcdic500,
		Ebcdic1140,
		Custom(String),
	}


	and exposing it through metadata.

	My recommendation for FFWB

	I would model every resource as one of three content models:

	enum ContentModel {
		Text,
		Binary,
		RecordOriented,
	}


	and:

	struct RecordAttributes {
		recfm: Recfm,
		lrecl: Option<u32>,
		blksize: Option<u32>,
		code_page: Option<String>,
	}


	Then a binary downloaded mainframe dataset can be:

	Imported into a catalog
	Associated with metadata
	Opened through the VFS
	Decoded from EBCDIC
	Reconstructed into FB/VB records
	Displayed as normal editor lines

	without altering the original binary representation.

	In fact, if you eventually want FileForgeWorkbench to emulate ISPF, SDSF, JES spool files, PDS members, sequential datasets, COBOL copybooks, ASA reports, and FileForge structured files, I would say record-oriented content plus code-page awareness should be treated as a core architectural feature of the VFS rather than an optional extension.

	
### New Bugs

	In the File explorer dialog / context,  
	if i am positioned on the command line, 
	if i press the tab key i should tab to the first catalog name in the file list area, 
	as I press tab i should tab through the list of files. if  I press tab key on a container name, it should expand to show the items it contains, As i press tab i should tab through the files.
	if i am positioned in the file list if i press an arrow key i shsould move in the direction of the Arrow key.
	if i press the arrow key, on a container it should not expand but i should move in the directions of the arrow key.
	if press Shift+ an Arrow Key the item gets highligted as selected, and the cursor move in the direction of the arrow key, selecting the next item as well.
	if i leave go of the shift button the selected items remain higligted as selected.
	if i press the ctrl key and an arrow key  move in the direction of the arrow key without changeing or adding anything to the selected items.
	if i keep the ctrl key pressed and press the space bar, the current items status toggles between selected and deselected as relevant.
	if i then press ctrl+c i copy the selected items. information.  

	if I then navigate to a new directory.  and press ctrl+v all the selected itesm should be copied to the new location.`
	
	if i paste into a file i am editing, the list of files should be pasted. 
	
Above requirements still not working
	
	Mainframe catalog is not presisting from one session to the next.
	On creating a new catalog, the default repository path for a mainfrmae catalog is empty.


    Buy using the mouse to selecting a file with the shift key pressed down, 

Break this task into smaller Tasks, so that i can monitor progress!	
Re-organise it into a proper request to analyse the following:
We seem to be struggeling to get the file tree to work correctly. We need to re-look at the requirements? 
There is an existing crate found at : https://crates.io/crates/egui-file-dialog, i believe this crate does largly what we need. 
We can refactor our reqiurements to meke use of this crate.
We should download this crate and its code, extract a full set of requirements from it. 
Amend our requirements to make full use of: https://crates.io/crates/egui-file-dialog requirements. 
Apply what makes sense to from our existing requirements to the new requirements.
And build making use of https://crates.io/crates/egui-file-dialog. 
Update our documents giving credit appropriatly to: https://crates.io/crates/egui-file-dialog    


but first i think we should examine this create and respecify our requirements to be more in line with meet this crates requirements  and that add adjust them slightly to meet FFWB better.


Tabbing is still not follwing the correct path, when opening the file econtext window. 
The tab position should be the command window, pressing tab should take us to the first Catalog name. 
Pressing tab again should take us to the first file in that catalog. 
After Reaching the last file in that catalog, pressing tab again should move to the next catalog name, and so on..
	
When looking at a catalogs propereties we dont see the repository path.  We should see the repository path. 
The VFS, should be able to determine a dataset's filename by looking at the dataset's catalog properties and the catalogs repository path to determione where the dats=aset resides.
	
Now getting: 'PAYROLL.EMPLOYEE': dataset file not found at C:\Users\A142367\AppData\Roaming\ffworkbench\catalogs\mainframe/Payroll\PAYROLL\EMPLOYEE.

This looks wrong: why VFS looking in "C:\Users\A142367\AppData\Roaming\ffworkbench\catalogs\mainframe/Payroll\PAYROLL\EMPLOYEE."  this does not look right?
The catlog properties are:  C:\Users\A142367\AppData\Roaming\ffworkbench\catalogs\mainframe/Payroll  this also looks wrong,  the path should probably be:
C:\Users\A142367\AppData\Roaming\ffworkbench\catalogs\mainframe\Payroll  
As far as i can see the path only extends to C:\Users\A142367\AppData\Roaming\ffworkbench  there is no folder here called \catalogs\ 

There is not option in the file explorere pannel to delete datasets...

'PAYROLL.EMPLOYEE': dataset file not found at C:\Users\A142367\AppData\Roaming\ffworkbench\catalogs\mainframe/Payroll\PAYROLL\EMPLOYEE.

I am still unhappy with the working of the Files window / Panel. 
which markdown files explicilty specify the desing and requirements of the files Window,  i think i need to review them carefully and look at rewordinf them.	
	
	
	
	
	
	Tabs are not independantly demarked, except some have square brackets arround them.  Is there not a way in EQU
	
PF1 . . . HELP        
PF2 . . . SPLIT       
PF3 . . . END         
PF4 . . . RETURN      
PF5 . . . RFIND       
PF6 . . . RCHANGE     
PF7 . . . UP          
PF8 . . . DOWN        
PF9 . . . SWAP        
PF10  . . LEFT        
PF11  . . RIGHT       
PF12  . . RETRIEVE    
PF13  . . HELP         
PF14  . . SPLIT        
PF15  . . END          
PF16  . . RETURN       
PF17  . . RFIND        
PF18  . . RCHANGE      
PF19  . . UP           
PF20  . . DOWN         
PF21  . . SWAP         
PF22  . . LEFT         
PF23  . . RIGHT        
PF24  . . RETRIEVE      

	Menu  Help                                                                   
	ssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss 
								Utility Selection Panel                             
	Option ===>                                                                    
																					
	1  Library     Compress or print data set.  Print index listing.  Print,       
					rename, delete, browse, edit or view members                  
	2  Data Set    Allocate, rename, delete, catalog, uncatalog, or display        
					information of an entire data set                             
	3  Move/Copy   Move, or copy members or data sets                              
	4  Dslist      Print or display (to process) list of data set names.           
					Print or display VTOC information                             
	5  Reset       Reset statistics for members of ISPF library                    
	6  Hardcopy    Initiate hardcopy output                                        
	8  Outlist     Display, delete, or print held job output                       
	9  Commands    Create/change an application command table                      
	11 Format      Format definition for formatted data Edit/Browse                
	12 SuperC      Compare data sets                             (Standard Dialog) 
	13 SuperCE     Compare data sets Extended                    (Extended Dialog) 
	14 Search-For  Search data sets for strings of data          (Standard Dialog) 
	15 Search-ForE Search data sets for strings of data Extended (Extended Dialog) 
	16 Tables      ISPF Table Utility                                              
	17 Udlist      Print or display (to process) z/OS UNIX directory list          
	
	zoom in and out with mouse  (also a menu option to set zoom value...)
	
	Examine this requirement and restate it after some analyses and consideration of the different file types, catalog types adn operating environmetns		

	
### Kiro	
	
	This project has been taken much further forward in Visual studio Code,  the project artifacts used are in Folder: "C:\workspace\VSC\FileForgeWorkbench"   
	Should we change the project to work from that folder or should we import everything from that folder to this folder?  
	What is the best way forward,  where this projects artifacts are now in 2 places ?

    
CIW2.master.cobol
CIW2.master.Jcl

CIW2
├─ MASTER 
|  ├─ COBOL
|  └─ JCL
├─ AsyncLoader
├─ WatchManager
├─ FilterEngine
├─ ContextMenuBuilder
└─ VFS Provider Interface