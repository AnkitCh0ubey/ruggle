use std::collections::HashMap;
use std::env;
use std::fs::{File, self};
use xml::reader::{EventReader, XmlEvent };
use std::path::{Path, PathBuf};
use xml::common::{Position, TextPosition};
use std::result::Result;
use std::process::ExitCode;

#[derive(Debug)]
struct Lexer<'a> {
   content: &'a [char],
}


impl<'a> Lexer<'a> {
   fn new(content: &'a [char]) -> Self{
      Self{ content }
   }


   //removes white space
   fn trim_left(&mut self){
      while self.content.len() > 0 && self.content[0].is_whitespace() {
         self.content = &self.content[1..]; 
      }
   }


   //predicate is used to pass a condition similar to modifiers in solidity
   fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char] 
   where 
      P: FnMut(&char) -> bool 
   {
      let mut n = 0;
      while n < self.content.len() && predicate(&self.content[n]){
         n += 1;
      }
      self.chop(n)

   }


   // to get the set of characters from lexer
   fn chop(&mut self, n: usize) -> &'a [char]{

      let token = &self.content[..n];
      self.content = &self.content[n..]; 
      token 
   }


   //this method is used to tokenize the string from the document, similar to how a StringTokenizer works in Java
   fn next_token(&mut self) -> Option<&'a [char]> {

      self.trim_left();

      if self.content.len() == 0 {
         return None
      }

      if self.content[0].is_alphabetic() { 
         return Some(self.chop_while(|x| x.is_alphanumeric())); // this closure determines the base of predicate
      } 
      
      if self.content[0].is_numeric() { 
         return Some(self.chop_while(|x| x.is_numeric()));
      }
      //for all the other symbols, we need to just pass it as they are so just pass 1
      Some(self.chop(1))
   }
}


// Since we implemented an iterator for lexer, it will give us a vector(array) of characters of the words.
impl<'a> Iterator for Lexer<'a> {
   type Item = &'a [char];
   
   fn next(&mut self) -> Option<Self::Item>{ // we get the next set of characters using the next function
      self.next_token()
   }
}

fn parse_entire_xml_file(file_path: &Path) -> Result<String, ()>{ 
   let file = File::open(file_path).map_err(|e|{
      eprintln!("ERROR: could not open the file {file_path}: {e}", file_path=file_path.display());
   })?;
   
   let er = EventReader::new(file);
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

type TermFrequency = HashMap<String, usize>;
type IndexTF = HashMap<PathBuf, TermFrequency>;


/*
fn main() -> io::Result<()> {
   let index_path = "index.json";
   let index_file = File::open(index_path)?;
   let result: IndexTF = serde_json::from_reader(index_file).expect("Uhh No something went wrong");
   println!("{index_path} contains {number} files", number = result.len());
   Ok(())
}

fn main2() -> io::Result<()>{

   let dir_path = "docs.gl/gl4";
   let dir = fs::read_dir(dir_path)?;
   let mut index_term_frequency = IndexTF::new();


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

fn tf_index_of_folder(dir_path: &Path, index_term_frequency: &mut IndexTF) -> Result<(),()> {

      let dir = fs::read_dir(dir_path).map_err(|err|{
         eprintln!("ERROR: could not open directory {dir_path} for indexing: {err}", dir_path = dir_path.display());
      })?;
      
      // IMPORTANT new lifetime thing that implements recursion 
      'next_file: for file in dir{
         let file = file.map_err(|e| {
            eprintln!("ERROR: could not read next file in the directory {dir_path} during indexing:{e}", dir_path = dir_path.display());
         })?;

         let file_path = file.path();

         let file_type = file.file_type().map_err(|e| {
            eprintln!("ERROR: could not determine type of file {file_path}: {e}", file_path = file_path.display());
         })?;

         if file_type.is_dir() {
            tf_index_of_folder(&file_path, index_term_frequency)?;
            continue 'next_file;
         }

         println!("Processing file: {:?}", &file_path);
         
         let content = match parse_entire_xml_file(&file_path) {
            Ok(content) => content.chars().collect::<Vec<_>>(),
            Err(()) => continue 'next_file,
         };
         

      let mut tf = TermFrequency::new();

      for token in Lexer::new(&content){
         let term = token.iter().map(|x| x.to_ascii_uppercase()).collect::<String>();

         if let Some(frequency) = tf.get_mut(&term) {
            *frequency += 1;
         } else {
            tf.insert(term, 1);
         }
      }

      index_term_frequency.insert(file_path, tf);    
   
   }

   Ok(())

}

fn save_tf_index(index_term_frequency: &IndexTF, filename: &str) -> Result<(),()>{
   println!("Saving {filename}");

   let file = File::create(filename).map_err(|err|{
      eprintln!("ERROR: couldn't create index file:{filename}: {}", err);
   })?;
   
   serde_json::to_writer_pretty(file, &index_term_frequency).map_err(|err| {
      eprintln!("ERROR: could not serialize index into file {filename}: {err}")
   })?;
   Ok(())
}

fn check_index(index_path: &str) -> Result<(),()> {
    println!("Reading {index_path} index file...");

    let index_file = File::open(index_path).map_err(|e|{
      eprintln!("ERROR: couldn't open index file: {index_path}: {e}");
    })?;

    let tf_index: IndexTF = serde_json::from_reader(index_file).map_err(|e|{
      eprintln!("ERROR: could not deserialize index from file {index_path}: {e}");
    })?;

    println!("{index_path} contains {count} files", count = tf_index.len());

    Ok(())
}

fn usage(program: &str){
   eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
   eprintln!("Subcommands:");
   eprintln!("    index <folder>          index the <folder> and save the index to index.json file");
   eprintln!("    search <index-file>     check how many documents are indexed in te file (searching is not implementred yet)");
   eprintln!("tiny-http");
}


fn entry() -> Result<(),()>{
   let mut args = env::args();
   let program = args.next().expect("path to program is provided");
   
   let subcommand = args.next().ok_or_else(|| {
      usage(&program);
      eprintln!("ERROR: No subcommand is provided");
   })?;

   match subcommand.as_str() {
      "index" => {
         let dir = args.next().ok_or_else(|| {
            usage(&program);
            eprintln!("ERROR: no directory is provided for {subcommand} subcommand")
         })?;
         let mut tf_index = IndexTF::new();
         tf_index_of_folder(Path::new(&dir), &mut tf_index)?;
         save_tf_index(&tf_index, "index.json")?;
      },
      "search" => {
         let index_path = args.next().ok_or_else(||{
            usage(&program);
            eprintln!("ERROR: No path to index is provided for {subcommand} subcommand");
         })?;
         check_index(&index_path)?;
      }
      _ => {
         usage(&program);
         eprintln!("ERROR: unknown subcommand {subcommand}");
         return Err(());
      }
   }
   Ok(())
}

fn main() -> ExitCode {
   match entry() {
      Ok(()) => ExitCode::SUCCESS,
      Err(()) => ExitCode::FAILURE,
   }
}