//! This module contains utilities for the initial input and quality control of the table cells
//!
//! Each table cell is modelled as having the ability to return a datatype and the contents as a String
//! We garantee that if these objects are created, then we are ready to create phenopackets.


#[derive(Debug)]
pub enum TableCellDataType {
    Title(TitleCell),
    PMID(PmidCell),
}



pub trait TableCell {
    fn value(&self) -> String;
}


pub struct HeaderItem {

}

/// This struct represents the contents of a cell of the Excel table that represents the title of a publication
#[derive(Debug)]
pub struct TitleCell {
    title: String,
}

impl TableCell for TitleCell {
    fn value(&self) -> String {
        self.title.clone()
    }
}

#[derive(Debug)]
pub struct PmidCell {
    pmid: String,
}

impl TableCell for PmidCell {
    fn value(&self) -> String {
        self.pmid.clone()
    }
}



pub struct DataFrame {


}