use chrono::Local;
use clap::Parser;
use futures::future::join_all;
use regex::Regex;
use rsdns::clients::{tokio::Client, ClientConfig};
use rsdns::{constants::Class, records::data::Aaaa, records::data::A};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, BufRead};
use std::net::SocketAddr;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Domain {
    name: String,
    resolved: bool,
}

impl Domain {
    fn new(name: String, resolved: bool) -> Self {
        Self {
            name: name,
            resolved: resolved,
        }
    }
}

struct DomainNames {
    domains: Vec<Domain>,
}

impl DomainNames {
    fn new() -> Self {
        Self {
            domains: Vec::<Domain>::new(),
        }
    }

    fn add(&mut self, domain: Domain) {
        self.domains.push(domain);
    }

    fn to_json(&mut self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.domains)
    }
}

struct DomainGenerator {
    path: String,
    top_level: String,
}

impl DomainGenerator {
    fn new(path: String, top_level: String) -> Self {
        Self {
            path: path,
            top_level: top_level,
        }
    }

    fn generate_domains(&self) -> io::Result<Vec<String>> {
        let file_path = Path::new(&self.path);
        let file = File::open(&file_path)?;
        self.words_to_domains(BufReader::new(file))
    }

    fn words_to_domains<R: BufRead>(&self, reader: R) -> io::Result<Vec<String>> {
        let mut domains = Vec::new();
        for line in reader.lines() {
            let name = line?;
            if name.is_empty() {
                continue;
            }
            let domain = format!("{}.{}", name, self.top_level);
            if self.valid_domain(&domain) {
                domains.push(domain);
            }
        }
        Ok(domains)
    }

    fn valid_domain(&self, domain: &String) -> bool {
        let regex_pattern = r"^[a-zA-Z0-9][a-zA-Z0-9-]{1,61}[a-zA-Z0-9]\.[a-zA-Z]{2,}$";
        let regex = Regex::new(regex_pattern).unwrap();
        regex.is_match(domain)
    }
}

struct AsyncDomainResolver {
    domains: Vec<String>,
    max_async_lookups: u32,
    resolved_domains: DomainNames,
}

impl AsyncDomainResolver {
    fn new(domains: Vec<String>) -> Self {
        Self {
            domains: domains,
            max_async_lookups: 20,
            resolved_domains: DomainNames::new(),
        }
    }

    fn resolve_domains(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        for domains in self.domains.chunks(self.max_async_lookups as usize) {
            let now = Local::now();
            println!("--- Verifying {} domains at {:?} ---", domains.len(), now);
            let verified_domains = rt.block_on(self.async_resolve_domains(domains));
            for domain in verified_domains {
                println!("domain: {}, resolved:{}", domain.name, domain.resolved);
                self.resolved_domains.add(domain);
            }
        }
    }

    fn as_json(&mut self) -> Result<String, serde_json::Error> {
        self.resolved_domains.to_json()
    }

    async fn async_resolve_domains(&self, domains: &[String]) -> Vec<Domain> {
        let mut futures = Vec::new();
        for domain in domains {
            let f = self.resolve_domain(domain.to_string());
            futures.push(f);
        }
        let results = join_all(futures).await;
        results
    }

    async fn resolve_domain(&self, qname: String) -> Domain {
        let ip_addr_and_port = "8.8.8.8:53";
        let nameserver: SocketAddr = ip_addr_and_port
            .parse()
            .expect("Unable to parse socket address");

        let config = ClientConfig::with_nameserver(nameserver);
        let mut client = Client::new(config)
            .await
            .expect("Unable to create DNS client");

        let rrset = client.query_rrset::<A>(qname.as_str(), Class::In).await;
        let rrset_ipv6 = client.query_rrset::<Aaaa>(qname.as_str(), Class::In).await;
        Domain::new(qname, rrset.is_ok() || rrset_ipv6.is_ok())
    }
}

// domain_resolver -n <names_list> <top_level_domain>
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path for the file with the names to use
    #[arg(short, long)]
    names_path: String,

    /// Top level domain, example: com
    #[arg(short, long, default_value = "com")]
    top_level: String,
    /// Path for the file with the names to use
    #[arg(short, long, default_value = "resolved_domains.json")]
    output_path: String,
}

fn main() {
    let args = Args::parse();

    let generator = DomainGenerator::new(args.names_path, args.top_level);

    match generator.generate_domains() {
        Ok(domains) => {
            let mut async_resolver = AsyncDomainResolver::new(domains);
            async_resolver.resolve_domains();
            if let Ok(json) = async_resolver.as_json() {
                let output_path = args.output_path;
                fs::write(output_path.clone(), json).expect("Unable to write file");
                println!("Output file: {}", output_path);
            } else if let Err(e) = async_resolver.as_json() {
                eprintln!("Failed to convert to JSON: {}", e);
            }
        }
        Err(e) => eprintln!("Error occurred: {}", e),
    }
}
