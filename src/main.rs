use std::str;
use std::env;
use std::path::Path;
use std::fs::{File, self};
use xml::reader::{EventReader, XmlEvent };
use xml::common::{Position, TextPosition};
use std::result::Result;
use std::process::ExitCode;
use std::io::{BufReader,BufWriter};


mod model;
use model::*;
mod server;

//takes the file path as the input and parses the entire XML file into String
fn parse_entire_xml_file(file_path: &Path) -> Result<String, ()>{ 
   let file = File::open(file_path).map_err(|e|{
      eprintln!("ERROR: could not open the file {file_path}: {e}", file_path=file_path.display());
   })?;
   
   let er = EventReader::new(BufReader::new(file));
   let mut content = String::new();
   
   for event in er.into_iter(){
      let event = event.map_err(|e|{
         let TextPosition{row, column} = e.position();
         let msg = e.msg();
         eprintln!("{file_path}:{row}:{column}: Error: {msg}", file_path = file_path.display());
      })?;

      if let XmlEvent::Characters(text) = event {
         content.push_str(&text);
         content.push(' ');
      }
   }
   Ok(content)
}

//function to index the folder
fn add_folder_to_model(dir_path: &Path, model: &mut Model) -> Result<(),()> {

      let dir = fs::read_dir(dir_path).map_err(|err|{
         eprintln!("ERROR: could not open directory {dir_path} for indexing: {err}", dir_path = dir_path.display());
      })?;
      
      // IMPORTANT 'next_file works like labelled loops in java 
      'next_file: for file in dir{
         let file = file.map_err(|e| {
            eprintln!("ERROR: could not read next file in the directory {dir_path} during indexing:{e}", dir_path = dir_path.display());
         })?;

         let file_path = file.path();

         let file_type = file.file_type().map_err(|e| {
            eprintln!("ERROR: could not determine type of file {file_path}: {e}", file_path = file_path.display());
         })?;

         if file_type.is_dir() {
            add_folder_to_model(&file_path, model)?;
            continue 'next_file;
         }

         println!("Processing file: {:?}", &file_path);
         
         let content = match parse_entire_xml_file(&file_path) {
            Ok(content) => content.chars().collect::<Vec<_>>(),
            Err(()) => continue 'next_file,
         };
         
      let mut tf = TermFrequency::new();// HashMap of term frequency of each file
      let mut n = 0;
      //to build term frequency of the model
      for term in Lexer::new(&content){
         if let Some(frequency) = tf.get_mut(&term) {
            *frequency += 1;
         } else {
            tf.insert(term, 1);
         }
         n += 1;
      }
      
      //to build document frequency (df) of the model
      for t in tf.keys(){
         if let Some(freq) = model.df.get_mut(t){
            *freq += 1;
         }
         else{
            model.df.insert(t.to_string(), 1);
         }
      }

      model.tfpf.insert(file_path, (n, tf));    
   
   }

   Ok(())

}

fn save_model(model: &Model, filename: &str) -> Result<(),()>{
   println!("Saving {filename}");

   let file = File::create(filename).map_err(|err|{
      eprintln!("ERROR: couldn't create index file:{filename}: {}", err);
   })?;
   
   serde_json::to_writer_pretty(BufWriter::new(file), &model).map_err(|err| {
      eprintln!("ERROR: could not serialize index into file {filename}: {err}")
   })?;

   Ok(())
}

fn usage(program: &str){
   eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
   eprintln!("Subcommands:");
   eprintln!("    index <folder>                         index the <folder> and save the index to index.json file");
   eprintln!("    search <index-file>                    to query the json (expected to be called directly from the UI");
   eprintln!("    total  <index-file>                     to get total number of files in the database");
   eprintln!("    serve  <index-file> [address]          to start the http server in a web interface");
}

/// main calls entry() and matches the Ok() and Err() types which entry() returns
fn entry() -> Result<(),()>{
   let mut args = env::args();
   let program = args.next().expect("path to program is provided");
   
   let subcommand = args.next().ok_or_else(|| {
      usage(&program);
      eprintln!("ERROR: No subcommand is provided");
   })?;

   match subcommand.as_str() {
      // index takes in an argument which is the directory path
      "index" => {
         let dir = args.next().ok_or_else(|| {
            usage(&program);
            eprintln!("ERROR: no directory is provided for {subcommand} subcommand")
         })?;
         let mut model:Model  = Default::default();
         add_folder_to_model(Path::new(&dir), &mut model)?;

         save_model(&model, "index-big.json")
      },
      
      "search" => {
         let index_path = args.next().ok_or_else(||{
            usage(&program);
            eprintln!("ERROR: No path to index is provided for {subcommand} subcommand");
         })?;

         let query = args.next().ok_or_else(||{
            usage(&program);
            eprintln!("ERROR: No query is provided !");
         })?.chars().collect::<Vec<_>>();

         let index_file = File::open(&index_path).map_err(|e|{
         eprintln!("ERROR: couldn't open index file: {index_path}: {e}");
         })?;
   
         let model: Model = serde_json::from_reader(index_file).map_err(|e|{
         eprintln!("ERROR: could not deserialize index from file {index_path}: {e}");
         })?;

         //calling to model's search function happens here!
         for(path, rank) in search(&model, &query).iter().take(20){
            println!("{path} {rank}", path = path.display());
         }
      Ok(())
      },

      // serve has two arguments 1: path to index file, 2: IP address (127.0.0.1:6969 is default)
      "serve" =>{
         let index_path = args.next().ok_or_else(||{
            usage(&program);
            eprintln!("ERROR: No path to index is provided for {subcommand} subcommand");
         })?;

         let index_file = File::open(&index_path).map_err(|e|{
            eprintln!("ERROR: couldn't open index file: {index_path}: {e}");
         })?;
      
         let model: Model = serde_json::from_reader(index_file).map_err(|e|{
            eprintln!("ERROR: could not deserialize index from file {index_path}: {e}");
         })?;

         let address = args.next().unwrap_or("127.0.0.1:6969".to_string());
         server::start(&address, &model)         
      },

      "total" =>{ 
         let path = args.next().ok_or_else(||{
            usage(&program);
            eprintln!("ERROR: No path to db file is provided");
         })?;
         let file = File::open(&path).map_err(|err|{
            eprintln!("ERROR: Couldn't open the file {path}: {err}");
         })?;
         let tf_index: TermFreqPerFile = serde_json::from_reader(file).map_err(|err|{
            eprintln!("ERROR: couldn't deserialize the data from file {path}: {err}");
         })?;
         println!("Total files in the database: {len}", len=tf_index.len());
         Ok(())
      }

      _ => {
         usage(&program);
         eprintln!("ERROR: unknown subcommand {subcommand}");
         Err(())
      }
   }
}

fn main() -> ExitCode {
   match entry() {
      Ok(()) => ExitCode::SUCCESS,
      Err(()) => ExitCode::FAILURE,
   }
}


/*
fn main() -> io::Result<()> {
   let index_path = "index.json";
   let index_file = File::open(index_path)?;
   let result: TermFreqPerFile = serde_json::from_reader(index_file).expect("Uhh No something went wrong");
   println!("{index_path} contains {number} files", number = result.len());
   Ok(())
}

fn main2() -> io::Result<()>{

   let dir_path = "docs.gl/gl4";
   let dir = fs::read_dir(dir_path)?;
   let mut index_term_frequency = TermFreqPerFile::new();


   for file in dir
   {
      let file_path = file?.path();
      println!("Processing file: {:?}", &file_path);
      let content = read_entire_xml_file(&file_path)?
         .chars()
         .collect::<Vec<_>>();

      let mut tf = TermFrequency::new(); 
      for token in Lexer::new(&content) 
      {
         let term = token.iter().map(|x| x.to_ascii_uppercase()).collect::<String>();
         if let Some(freq) = tf.get_mut(&term) {
            *freq += 1;
         }else {
            tf.insert(term, 1);
         }
      }
      let mut stats = tf.iter().collect::<Vec<_>>();
      stats.sort_by_key(|(_,f)| *f);
      stats.reverse();
      index_term_frequency.insert(file_path, tf);     
   }
   let index_path = "index.json";
   let index_file = File::create(index_path)?;
   println!("Saving {index_path}....");
   serde_json::to_writer_pretty(index_file, &index_term_frequency).expect("Serde is working fine");
   
   Ok(())
}
*/