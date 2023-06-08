use chrono::Local;
use futures::executor::block_on;
use futures::future::join_all;
use futures::Future;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, BufRead};
use std::path::Path;
use std::pin::Pin;
use trust_dns_resolver::config::ResolverConfig;
use trust_dns_resolver::config::ResolverOpts;
use trust_dns_resolver::Resolver;

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
            let domain = format!("{}.{}", line?, self.top_level);
            domains.push(domain);
        }
        Ok(domains)
    }
}

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

struct DomainList {
    domains: Vec<Domain>,
}

impl DomainList {
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

struct AsyncDomainResolver {
    domains: Vec<String>,
    max_async_lookups: u32,
    resolved_domains: DomainList,
}

impl AsyncDomainResolver {
    fn new(domains: Vec<String>) -> Self {
        Self {
            domains: domains,
            max_async_lookups: 20,
            resolved_domains: DomainList::new(),
        }
    }

    fn resolve_domains(&mut self) {
        for domains in self.domains.chunks(self.max_async_lookups as usize) {
            let now = Local::now();
            println!(
                "--- Verifying {} domains at {:?} ---",
                self.max_async_lookups, now
            );

            let verified_domains = block_on(self.async_resolve_domains(domains));
            for domain in verified_domains {
                println!("domain: {}, resolved:{}", domain.name, domain.resolved);
                self.resolved_domains.add(domain);
            }
        }
    }

    fn save_to_json(&mut self) {
        match self.resolved_domains.to_json() {
            Ok(json) => {
                let filename = self.filename();
                fs::write(filename.clone(), json).expect("Unable to write file");
                println!("Output: {}", filename);
            }
            Err(e) => eprintln!("Failed to convert to JSON: {}", e),
        }
    }

    fn filename(&mut self) -> String {
        use random_string::{Charset, Charsets, RandomString};
        let charset = Charset::from_charsets(Charsets::LettersLowercase);
        let prefix = RandomString::generate(6, &charset);
        let now = Local::now();
        format!("{}_{}.json", prefix, now.timestamp())
    }

    async fn async_resolve_domains(&self, domains: &[String]) -> Vec<Domain> {
        let mut futures: Vec<Pin<Box<dyn Future<Output = Domain> + Send>>> = Vec::new();
        for domain in domains {
            let f = Box::pin(self.resolve_domain(domain.to_string()));
            futures.push(f);
        }
        let results = join_all(futures).await;
        results
    }

    async fn resolve_domain(&self, domain: String) -> Domain {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
        let response = resolver.lookup_ip(format!("{}.", domain).as_str());
        let resolved = match response {
            Ok(_) => true,
            Err(_) => false,
        };
        Domain::new(domain, resolved)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <domain_list_file> <top_level_domain>", args[0]);
        return;
    }

    let path = args[1].clone();
    let top_level = args[2].clone();
    let generator = DomainGenerator::new(path, top_level);

    match generator.generate_domains() {
        Ok(domains) => {
            let mut async_resolver = AsyncDomainResolver::new(domains);
            async_resolver.resolve_domains();
            async_resolver.save_to_json();
        }
        Err(e) => eprintln!("Error occurred: {}", e),
    }
}
