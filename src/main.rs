// Copyright 2018 witchof0x20
/*  This file is part of freeotp_migrate.

    freeotp_migrate is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    freeotp_migrate is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with freeotp_migrate.  If not, see <http://www.gnu.org/licenses/>.
*/
//Argument parsing
extern crate argparse;
//XML Parsing
extern crate xml;
// Json parsing
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
// URI encoding
extern crate url;
// Base32 encoding
extern crate base32;
// QR code generation
#[cfg(feature = "qrcode_create")]
extern crate image;
#[cfg(feature = "qrcode_create")]
extern crate qrcode;
//Include the Token object
mod token;

// Argument parsing
use argparse::{ArgumentParser, Store};
// Only import this if qr code support is enabled
#[cfg(feature = "qrcode_create")]
use argparse::StoreTrue;
// File reading
#[cfg(feature = "qrcode_create")]
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
// XML Parsing
use xml::reader::{EventReader, XmlEvent};
// Use token object
use token::Token;

fn main() {
    // The path to tokens.xml
    let mut tokens_filename: String = "tokens.xml".to_string();
    // Only create these if qr code support is enabled
    // Whether to write qr codes
    #[cfg(feature = "qrcode_create")]
    let mut write_qr: bool = false;
    // Where to write qr codes
    #[cfg(feature = "qrcode_create")]
    let mut qr_folder: String = ".".to_string();
    // Parse arguments
    // Argument parsing should be in its own scope
    {
        // Initialize parser
        let mut arg_parser = ArgumentParser::new();
        // Set program description
        arg_parser
            .set_description("Parses a tokens.xml file from FreeOTP and generates new 2FA links");
        // Set up a flag for the filename
        arg_parser.refer(&mut tokens_filename).add_option(
            &["-i", "--input-file"],
            Store,
            "path to the tokens.xml file",
        );
        // Only if qr support is built
        #[cfg(feature = "qrcode_create")]
        {
            // Set up flag for writing qr codes
            arg_parser.refer(&mut write_qr).add_option(
                &["-q", "--generate-qr"],
                StoreTrue,
                "generate QR codes",
            );
            // Set up flag for qr code output folder
            arg_parser.refer(&mut qr_folder).add_option(
                &["-o", "--qr-output"],
                Store,
                "folder to store the QR code images in. Default is current directory",
            );
        }
        // Parse arguments
        arg_parser.parse_args_or_exit();
    }
    // Open the tokens file
    let tokens_file = match File::open(&tokens_filename) {
        Ok(file_handle) => file_handle,
        Err(error) => panic!("Error opening {}: {}", &tokens_filename, error),
    };
    // Convert the file handle to a buffered reader
    let tokens_file = BufReader::new(tokens_file);
    // Get a buffered read on the file
    let config_parser = EventReader::new(tokens_file);
    // Iterate over the xml
    for xml_event in config_parser {
        match xml_event {
            // Read the token data
            Ok(XmlEvent::Characters(characters)) => {
                // Don't parse the list entry
                if characters.chars().next().unwrap() == '{' {
                    // Parse each entry as json
                    let token: Token = match serde_json::from_str(&characters) {
                        Ok(token) => token,
                        Err(error) => panic!("Error parsing token object: {}", error),
                    };
                    // Print the token as a URI
                    println!("{}", token);
                    // Only if qr support is enabled
                    #[cfg(feature = "qrcode_create")]
                    {
                        if write_qr {
                            // Create an output path
                            let mut output_path = PathBuf::from(&qr_folder);
                            output_path.push(format!("{}.png", token.label));
                            // Save the qr image
                            match output_path.to_str() {
                                Some(path) => token.save_qr(path.to_string()),
                                None => panic!("Error creating filename"),
                            }
                        }
                    }
                }
            }
            // Panic on xml parsing error
            Err(error) => panic!("XML parsing error: {}", error),
            // Ignore other xml events
            _ => {}
        }
    }
}
