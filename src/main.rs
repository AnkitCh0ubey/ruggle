use std::collections::HashMap;
use std::io;
use std::fs::{File, self};
use xml::reader::{EventReader, XmlEvent };
use std::path::{Path, PathBuf};


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


fn index_document(_doc_content: &str) -> HashMap<String, usize>{
   todo!("Yet to be implemented");
}


fn read_entire_xml_file<P: AsRef<Path>>(file_path: P) -> io::Result<String>{ 
   let file = File::open(file_path)?;
   
   let er = EventReader::new(file);
   let mut content = String::new();
   
   for event in er.into_iter(){
      if let XmlEvent::Characters(text) = event.expect("Will wrap the xml error and the io error together"){
         content.push_str(&text);
      }
   }
   Ok(content)
}

type TermFrequency = HashMap<String, usize>;
type IndexTF = HashMap<PathBuf, TermFrequency>;

fn main() -> io::Result<()>{

   let dir_path = "docs.gl/gl4";
   let dir = fs::read_dir(dir_path)?;
   let n = 20;
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
         }
         else {
            tf.insert(term, 1);
         }
      }
   
      let mut stats = tf.iter().collect::<Vec<_>>();
      stats.sort_by_key(|(_,f)| *f);
      stats.reverse();
      
      index_term_frequency.insert(file_path, tf);
      
   }

   for (path, tf) in index_term_frequency
   {
      println!("File: {path:?} contains {number} unique tokens.", number = tf.len());
   }
   
   Ok(())
}