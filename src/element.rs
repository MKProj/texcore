use crate::Level::*;
use crate::TextType::*;
use crate::Type::*;
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
#[cfg(feature = "compile")]
use tectonic::latex_to_pdf;

/// Converts a struct to a string
pub trait Tex {
    fn to_latex_string(&self) -> String;
}

impl Tex for Any {
    fn to_latex_string(&self) -> String {
        match self.type_ {
            T_Input => Input::new(&self.value, self.level).to_latex_string(),
            T_Package => Package::new(&self.value).to_latex_string(),
            T_Part => Part::new(&self.value).to_latex_string(),
            T_Chapter => Chapter::new(&self.value).to_latex_string(),
            T_Header => {
                if self.header_level.is_some() {
                    Header::new(&self.value, self.header_level.unwrap()).to_latex_string()
                } else {
                    Header::new(&self.value, 1).to_latex_string()
                }
            }
            T_Paragraph => Paragraph::new(&self.value).to_latex_string(),
            T_Text => match self.text_type {
                None => Text::new(&self.value, Normal).to_latex_string(),
                Some(t) => Text::new(&self.value, t).to_latex_string(),
            },
            T_Environment => {
                let mut env = Environment::new(&self.value);
                env.set_elements(self.elements.clone().unwrap());
                env.to_latex_string()
            }
            T_Custom => self.value.clone(),
            T_List => match self.list_type {
                None => {
                    List::new(ListType::Itemized, self.items.clone().unwrap()).to_latex_string()
                }
                Some(s) => List::new(s, self.items.clone().unwrap()).to_latex_string(),
            },
            T_Item => Item::new(&self.value).to_latex_string(),
        }
    }
}

impl Tex for Environment {
    fn to_latex_string(&self) -> String {
        let begin = format!(r"\begin{{{}}}", &self.name);
        let end = format!(r"\end{{{}}}", &self.name);
        let strings = vec![begin, self.inner_latex_string(), end];
        strings.join("\n")
    }
}

impl Tex for Custom {
    fn to_latex_string(&self) -> String {
        self.value.clone()
    }
}

impl Tex for Input {
    fn to_latex_string(&self) -> String {
        format!(r"\input{{{}}}", &self.file_name)
    }
}

impl Tex for Package {
    fn to_latex_string(&self) -> String {
        format!(r"\usepackage{{{}}}", &self.pkg)
    }
}

impl Tex for Part {
    fn to_latex_string(&self) -> String {
        format!(r"\part{{{}}}", &self.name)
    }
}

impl Tex for Chapter {
    fn to_latex_string(&self) -> String {
        format!(r"\chapter{{{}}}", &self.name)
    }
}

impl Tex for Header {
    fn to_latex_string(&self) -> String {
        if self.header_level == 1 as u8 {
            format!(r"\section{{{}}}", &self.name)
        } else if self.header_level > 1 as u8 {
            let mut result = r"\".to_string();
            let mut count = 0;
            while count < self.header_level {
                result.push_str("sub");
                count += 1;
            }
            result.push_str(&format!("section{{{}}}", &self.name));
            result
        } else {
            "header number error".to_string()
        }
    }
}

impl Tex for Paragraph {
    fn to_latex_string(&self) -> String {
        format!(r"\paragraph{{{}}}", &self.content)
    }
}

impl Tex for Text {
    fn to_latex_string(&self) -> String {
        return match &self.type_ {
            Bold => format!(r"\textbf{{{}}}", &self.content),
            Italics => format!(r"\textit{{{}}}", &self.content),
            Normal => format!(r"{}", &self.content),
            Math => format!("${}$", &self.content),
            Par => format!(r"\par {{{}}}", &self.content),
        };
    }
}

impl Tex for Item {
    fn to_latex_string(&self) -> String {
        format!(r"\item {{{}}}", &self.name)
    }
}

impl Tex for List {
    fn to_latex_string(&self) -> String {
        let list = match &self.type_ {
            ListType::Itemized => "itemize",
            ListType::Enumerated => "enumerate",
        };
        let begin = format!(r"\begin{{{}}}", list);
        let end = format!(r"\end{{{}}}", list);

        let mut result = Vec::new();
        result.push(begin);
        for i in &self.items {
            result.push(i.to_latex_string())
        }
        result.push(end);
        result.join("\n")
    }
}

impl Tex for Metadata {
    fn to_latex_string(&self) -> String {
        let doc_class = format!(
            r"\documentclass[{}pt, letterpaper]{{{}}}",
            &self.fontsize, &self.doc_class
        );
        let title = format!(r"\title{{{}}}", &self.title);
        let author = format!(r"\author{{{}}}", &self.author);
        let date = format!(r"\date{{{}}}", &self.date);
        let result = vec![doc_class, title, author, date];
        result.join("\n")
    }
}

impl From<Part> for Element<Any> {
    fn from(part: Part) -> Self {
        let any = Any {
            value: part.name,
            type_: T_Part,
            level: None,
            header_level: None,
            text_type: None,
            list_type: None,
            items: None,
            elements: None,
        };
        Element::new(any, T_Part, Document)
    }
}

impl From<Chapter> for Element<Any> {
    fn from(chapter: Chapter) -> Self {
        let any = Any {
            value: chapter.name,
            type_: T_Chapter,
            level: None,
            header_level: None,
            text_type: None,
            list_type: None,
            items: None,
            elements: None,
        };
        Element::new(any, T_Chapter, Document)
    }
}

impl From<Header> for Element<Any> {
    fn from(header: Header) -> Self {
        let any = Any {
            value: header.name,
            type_: T_Header,
            level: None,
            header_level: Some(header.header_level),
            text_type: None,
            list_type: None,
            items: None,
            elements: None,
        };
        Element::new(any, T_Header, Document)
    }
}

impl From<Paragraph> for Element<Any> {
    fn from(paragraph: Paragraph) -> Self {
        let any = Any {
            value: paragraph.content,
            type_: T_Paragraph,
            level: None,
            header_level: None,
            text_type: None,
            list_type: None,
            items: None,
            elements: None,
        };
        Element::new(any, T_Paragraph, Document)
    }
}

impl From<Text> for Element<Any> {
    fn from(text: Text) -> Self {
        let any = Any {
            value: text.content,
            type_: T_Text,
            level: None,
            header_level: None,
            text_type: Some(text.type_),
            list_type: None,
            items: None,
            elements: None,
        };
        Element::new(any, T_Text, Document)
    }
}

impl From<Package> for Element<Any> {
    fn from(package: Package) -> Self {
        let any = Any {
            value: package.pkg,
            type_: T_Package,
            level: None,
            header_level: None,
            text_type: None,
            list_type: None,
            items: None,
            elements: None,
        };
        Element::new(any, T_Package, Packages)
    }
}

impl From<Input> for Element<Any> {
    fn from(input: Input) -> Self {
        let any = Any {
            value: input.file_name,
            type_: T_Input,
            level: input.level,
            header_level: None,
            text_type: None,
            list_type: None,
            items: None,
            elements: None,
        };
        Element::new(any, T_Input, input.level.unwrap())
    }
}

impl From<Environment> for Element<Any> {
    fn from(environment: Environment) -> Self {
        let any = Any {
            value: environment.name,
            type_: T_Environment,
            level: None,
            header_level: None,
            text_type: None,
            list_type: None,
            items: None,
            elements: Some(environment.elements),
        };
        Element::new(any, T_Environment, Document)
    }
}

impl From<Custom> for Element<Any> {
    fn from(custom: Custom) -> Self {
        let any = Any {
            value: custom.value,
            type_: T_Custom,
            level: Some(custom.level),
            header_level: None,
            text_type: None,
            list_type: None,
            items: None,
            elements: None,
        };
        Element::new(any, T_Custom, custom.level)
    }
}

/// A latex element
#[derive(PartialOrd, PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct Element<T: Tex> {
    pub(crate) value: T,
    type_: Type,
    level: Level,
}

impl<T: Tex> Element<T> {
    /// Creates a new Element
    pub fn new(value: T, type_: Type, level: Level) -> Self {
        Self {
            value,
            type_,
            level,
        }
    }
}

/// A linked list of elements
#[derive(Debug, Clone, Deserialize, Serialize, PartialOrd, PartialEq)]
pub struct ElementList<T: Tex> {
    metadata: Metadata,
    list: LinkedList<Element<T>>,
}

impl ElementList<Any> {
    /// Creates a new empty list
    pub fn new(
        author: &str,
        date: &str,
        title: &str,
        fontsize: u8,
        doc_class: &str,
        maketitle: bool,
    ) -> Self {
        Self {
            metadata: Metadata::new(author, date, title, fontsize, doc_class, maketitle),
            list: LinkedList::new(),
        }
    }
    /// Adds in `\newpage` text as next element in the list
    pub fn add_newpage(&mut self) {
        let text = Text::new(r"\newpage", Normal);
        self.push(text.into());
    }
    /// Pushes an element to the end of the list
    pub fn push(&mut self, element: Element<Any>) {
        self.list.push_back(element)
    }
    /// Pushes an element vector into the list
    pub fn push_array(&mut self, element_vec: Vec<Element<Any>>) {
        for element in element_vec {
            self.push(element)
        }
    }
    /// Pops an element at the end of the list
    pub fn pop(&mut self) -> Option<Element<Any>> {
        self.list.pop_back()
    }
    /// Pushes element to the front of the list
    pub fn fpush(&mut self, element: Element<Any>) {
        self.list.push_front(element)
    }
    /// Pops an element in the front of the list
    pub fn fpop(&mut self) -> Option<Element<Any>> {
        self.list.pop_front()
    }
    /// Sorts the list going from Meta, Packages, Document and returns new list
    pub fn sort(self) -> Self {
        let mut elements = Vec::new();
        for i in self.list {
            elements.push(i)
        }
        let mut new = LinkedList::new();
        let meta = self.metadata.clone();

        elements.sort_by(|a, b| a.level.partial_cmp(&b.level).unwrap());

        for i in elements {
            new.push_back(i);
        }
        Self {
            metadata: meta,
            list: new.into(),
        }
    }
    /// Walks the list and returns a combined latex string
    pub fn to_latex_string(self) -> String {
        let mut meta = Vec::new();
        meta.push(self.metadata.to_latex_string().to_owned());
        let mut packages = Vec::new();
        let mut document = Vec::new();
        document.push(r"\begin{document}".to_owned());
        if self.metadata.maketitle {
            document.push(r"\maketitle".to_owned());
        }
        let sort = self.sort();
        for i in &sort.list {
            match i.level {
                Document => document.push(i.value.to_latex_string()),
                Packages => packages.push(i.value.to_latex_string()),
                Meta => meta.push(i.value.to_latex_string()),
            }
        }
        document.push(r"\end{document}".to_owned());
        let mut result = Vec::new();
        result.push(meta.join("\n"));
        result.push(packages.join("\n"));
        result.push(document.join("\n"));
        result.join("\n")
    }
    /// Walks the list and returns a split latex string separating Packages level
    pub fn to_latex_split_string(self) -> (String, String) {
        let mut meta = Vec::new();
        meta.push(self.metadata.to_latex_string().to_owned());
        let mut packages = Vec::new();
        let mut document = Vec::new();
        document.push(r"\begin{document}".to_owned());
        if self.metadata.maketitle {
            document.push(r"\maketitle".to_owned());
        }
        let sort = self.sort();
        for i in &sort.list {
            match i.level {
                Document => document.push(i.value.to_latex_string()),
                Packages => packages.push(i.value.to_latex_string()),
                Meta => meta.push(i.value.to_latex_string()),
            }
        }
        document.push(r"\end{document}".to_owned());
        let mut result = Vec::new();
        result.push(meta.join("\n"));
        result.push(document.join("\n"));
        (result.join("\n"), packages.join("\n"))
    }
    /// Writes files from turning list into string
    pub fn write(
        self,
        path1: PathBuf,
        path2: Option<PathBuf>,
        split: bool,
    ) -> Result<(), std::io::Error> {
        let (seg1, seg2) = if split {
            self.to_latex_split_string()
        } else {
            (self.to_latex_string(), "".to_string())
        };
        let mut file1 = File::create(path1)?;
        file1.write_all(seg1.as_bytes())?;
        if let Some(path2) = path2 {
            let mut file2 = File::create(path2)?;
            file2.write_all(seg2.as_bytes())?;
        }
        Ok(())
    }
    #[cfg(feature = "compile")]
    /// Compiles the list into a pdf file
    pub fn compile(self, path: PathBuf) -> Result<(), Error> {
        let mut file = File::create(path)?;
        let latex = self.to_latex_string();
        let pdf = latex_to_pdf(&latex)?;
        file.write_all(&pdf)?;
        Ok(())
    }
    /// Prints the whole tex source code
    pub fn cat(self) {
        println!("{}", self.to_latex_string());
    }
    /// Returns &self.list to Vec<Any>
    pub fn list_to_vec(&self) -> Vec<Element<Any>> {
        let mut vec = Vec::new();
        for l in &self.list {
            vec.push(l.to_owned())
        }
        vec
    }
    /// returns the list's metadata
    pub fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }
}

impl Default for ElementList<Any> {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            list: LinkedList::new(),
        }
    }
}
