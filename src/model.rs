use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

pub type TermFrequency = HashMap<String, usize>;
pub type TermFreqPerFile = HashMap<PathBuf, (usize, TermFrequency)>;
pub type DocFrequency = HashMap<String, usize>;

#[derive(Default, Serialize, Deserialize)]
pub struct Model {
   pub tfpf: TermFreqPerFile,
   pub df: DocFrequency
}

// tf is the term frequency
pub fn compute_tf(current_term:  &str, n: usize, freq_map: &TermFrequency) -> f32 {
   let n = n as f32; //sum of the frequency of all the terms stored in freq_map
   let cf = freq_map.get(current_term).cloned().unwrap_or(0) as f32; //gets the frequency of the current term stored in freq_map
   cf/n
 }
 
 //idf is the inverse document frequency
 pub fn compute_idf(current_term: &str, n: usize, df: &DocFrequency) -> f32 {
   let n = n as f32;
   let df = df.get(current_term).cloned().unwrap_or(1) as f32;
   (n/df).ln()
 }
 
 pub struct Lexer<'a> {
    content: &'a [char],
 }
 
 impl<'a> Lexer<'a> {
   pub fn new(content: &'a [char]) -> Self{
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
   pub fn next_token(&mut self) -> Option<String> {

      self.trim_left();

      if self.content.len() == 0 {
         return None
      }

      if self.content[0].is_alphabetic() { 
         return Some(self.chop_while(|x| x.is_alphanumeric()).iter().map(|x| x.to_ascii_uppercase()).collect::<String>()); // this closure determines the base of predicate
      } 
      
      if self.content[0].is_numeric() { 
         return Some(self.chop_while(|x| x.is_numeric()).iter().collect());
      }
      //for all the other symbols, we need to just pass it as they are so just pass 1 
      Some(self.chop(1).iter().collect())
   }
 }
 
 // Since we implemented an iterator for lexer, it will give us a vector(array) of words.
 impl<'a> Iterator for Lexer<'a> {
   type Item = String;
   
   fn next(&mut self) -> Option<Self::Item>{ // we get the next set of characters using the next function
      self.next_token()
   }
 }
 pub fn search<'a>(model: &'a Model, query: &'a [char]) -> Vec<(&'a Path, f32)>{
   let mut result= Vec::<(&Path, f32)>::new();
   let token = Lexer::new(&query).collect::<Vec<_>>();
   for (path, (n,tf_table)) in &model.tfpf{
      let mut rank = 0f32;
      for token in &token {
            rank += compute_tf(&token, *n, &tf_table) * compute_idf(&token, model.tfpf.len(), &model.df);
      }
      result.push((path, rank));
   }
      result.sort_by(|(_, rank1), (_, rank2)| rank1.partial_cmp(rank2).unwrap());
      result.reverse();
      result
}