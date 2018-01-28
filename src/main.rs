use std::env;
use std::net::SocketAddr;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::fs::File;
use std::net::Shutdown;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::time::{Duration, SystemTime};

extern crate num_cpus;

const MAX_ONE_SHOT:u32 = 64;
const MAX_ITERATIONS:u32 = 10;
const CACHE_SIZE:usize = 5;
const MAX_CACHE_FSIZE: u64 = 10 * 1024;
const MAX_BUFF_SIZE: usize = 1 * 1024 * 1024;

pub struct ProducerWorker {
    id: u32,
    producer: Arc<Mutex<Sender<TcpStream>>>,
}

pub struct LRUCache {
	size: usize,
	dq: VecDeque<String>,
	content_map: HashMap<String, [u8; MAX_CACHE_FSIZE as usize]>,
	mtime_map: HashMap<String, SystemTime>,
	contents : [u8; MAX_CACHE_FSIZE as usize],
}

pub struct Worker {
    id: u32,
    producer: Arc<Mutex<Sender<TcpStream>>>,
    consumer: Arc<Mutex<Receiver<TcpStream>>>, //receiver is the client who we want to deliver data
    cache: LRUCache,
    //core: JoinHandle<()>,
    //TODO Add a file object which tracks the number of bytes read
}

pub struct Request {
    file: File,
    read: u64,
    done: bool,
    stream: TcpStream,
    len: u64,
    filename: String,
}

pub struct Core {
    base_path: String,
    max_tick: u32,
    pending: Vec<Request>,
    cache: LRUCache,
}


impl Core {
    fn tick(&mut self) {
        let mut iteration = 0;
        let mut processed = 0;

        while iteration < 1 && processed < MAX_ONE_SHOT {
            let mut it = self.pending.iter_mut();

            //println!("Starting reactor tick: {}", iteration);
        
            while let Some(cur) = it.next() {
                if cur.done {
                    continue;
                }
                let mut cur_file = &cur.file;
                let cur_bytes_read = cur.read;
                let mut len = cur.len;

                //println!("Processing: {}, read:{}", &cur.filename[..], cur.read);
                if cur_bytes_read == 0 && len == 0 {
                    let metadata = match cur_file.metadata() {
                        Ok(m) => m,
                        Err(e) => {
                            continue;
                            //TODO: mark as error
                        },
                    };
                    len = metadata.len();
                    //println!("The length of file:{} is {}",&cur.filename[..], len);
                    cur.len = len;
                    if metadata.is_dir() {
                    	//TODO: Handle Directory	
                    }		
					//Call Cache.query instead of adding request.
					
					if cur.len < MAX_CACHE_FSIZE {
						let fname = format!("{}{}", self.base_path, cur.filename);						
						let mtime = match metadata.modified() {
							Ok(t) => {								
								//println!("Modified at {:?}", t);
								t
							}, 
							Err(_) => {
								//println!("Not supported on this platform");
								SystemTime::now()
							}
						};
						println!("Querying Cache for file{} size{}", fname, cur.len);
						let contents = self.cache.query(fname, mtime);
						cur.stream.write(&contents).unwrap();
	                    cur.stream.flush().unwrap();
	                    cur.done = true;
                        cur.stream.shutdown(Shutdown::Both).unwrap();
                        processed += 1;
				        if processed >= 10 {
				            break;
				        }
				        continue;
					}
					
                }
                let mut buffer = [0; MAX_BUFF_SIZE];
                let bytes = match cur_file.read(&mut buffer) {
                    Ok(size) => size,
                    Err(e) => {
                        println!("ERROR: {:?}", e);
                        1000 * 1000 * 1000
                    },
                };
                //println!("File:{}, Bytes read:{}", &cur.filename[..], bytes);
                if bytes == 0 { //we are done reading ?
                    cur.done = true;
                    cur.stream.flush().unwrap();
					//println!("Done Reading file{}, closing connection", cur.filename);
                    cur.stream.shutdown(Shutdown::Both).unwrap();
                    
                    // Call shutdown cur.stream
                } else if bytes <= MAX_BUFF_SIZE {
                    cur.stream.write(&buffer[0..bytes]).unwrap();
                    cur.stream.flush().unwrap();
                    //let seeked = cur_file.seek(SeekFrom::Current(bytes as i64)).unwrap();
                    //println!("Seeked to: {}", seeked);
                } else {
                    println!("ERROR: {}", &cur.filename[..]); //TODO
                }
                cur.read += bytes as u64;
                //println!("File:{}, Total Bytes read:{}", &cur.filename[..], cur.read);
                
                processed += 1;
                if processed >= 10 {
                    break;
                }
            }
            //println!("Done reactor tick:{}", iteration);
            iteration += 1;

        
        }

        self.pending.retain(|x| !x.done);
        //println!("Size of internal queue:{}", self.pending.len());

        //self.pending.into_iter().filter(|x| x.done);
        
    }

    fn add(&mut self, request: Request) {
        self.pending.push(request);
    }
}


impl LRUCache {

	pub fn new(size: usize) -> LRUCache {    
		let dq = VecDeque::with_capacity(size);
		let contents : [u8; MAX_CACHE_FSIZE as usize] = [0; MAX_CACHE_FSIZE as usize];
        let content_map : HashMap<String, [u8; MAX_CACHE_FSIZE as usize]> = HashMap::new();
        let mtime_map: HashMap<String, SystemTime> = HashMap::new();
        let cache = LRUCache {
            size: 0,
            dq,
            content_map,
            mtime_map,
            contents,
        };        
        cache
    }
    
	fn insert_new_data(&mut self, filename: String, mtime: SystemTime) -> [u8; MAX_CACHE_FSIZE as usize] {
    	let mut buffer : [u8; MAX_CACHE_FSIZE as usize] = [0; MAX_CACHE_FSIZE as usize];
		let mut f = File::open(&filename).unwrap();	
			
		let bytes = match f.read(&mut buffer[..]) 
					{
		                Ok(size) => size,
		                Err(e) => {
		                    println!("ERROR: {:?}", e);
		                    0
		                },
                	};

		self.content_map.insert(filename.clone(), buffer);
		self.mtime_map.insert(filename.clone(), mtime);
		self.dq.push_back(filename.clone());
		if self.dq.len() <= CACHE_SIZE {
			self.size += 1;
		} 
		else {
			//Remove first element and insert new element at back		
			let x = self.dq.pop_front();
			let fname = match x {
				Some(f) => f,
				None => "NA".to_string(),
			};
			self.content_map.remove(&fname);				
			self.mtime_map.remove(&fname);
		}			
		//println!("Returning: {}", contents);
		return buffer;
	}
	
	fn refresh_contents(&mut self, filename: String, mtime: SystemTime) -> [u8; MAX_CACHE_FSIZE as usize] {
		let index = match self.dq.iter().position(|ref x| x.to_string() == filename) {
			Some(id) => {
				//println!("Index: {}", id);
				id
			}
			None => {
				//println!("Error couldn't find index in DQ{:?}", self.dq);
				0
			}
		};
		self.dq.remove(index);
		self.content_map.remove(&filename);
		self.mtime_map.remove(&filename);
		return self.insert_new_data(filename, mtime);
	}
    pub fn query(&mut self, filename: String, mtime: SystemTime) -> [u8; MAX_CACHE_FSIZE as usize] {
    	//let mut contents : [u8; MAX_CACHE_FSIZE as usize] = [0; MAX_CACHE_FSIZE as usize];
    	//println!("Cache:: {} {:?}", self.size, self.dq);
    	//Check the map if entry is available.    	
		if self.content_map.contains_key(&filename) {
			let old_mtime: SystemTime = match self.mtime_map.get(&filename) {
				Some(x) => {
					//println!("MtimeGet {}: {:?}", filename, x);
					x.clone()
				},
				None => {
					//println!("Error {} is not present in mtime_map!!!.", filename);
					SystemTime::now()
				}
			};
			//println!("Old Time: {:?}, mtime {:?}", old_mtime, mtime);
			if old_mtime.eq(&mtime) == true {
				//file not modified. 
				//println!("Content is still fresh...");
			}
			else {
				//file got modified. need to update contents.
				//println!("Content is outdated...updating...");
				//remove the file from content_map, dq, mtime_map
				return self.refresh_contents(filename, mtime);
			}
			//Return the data & Update DQ			
			self.contents = match self.content_map.get(&filename) {
				Some(c) => {
					//println!("Cmap Get {}: {}", filename, c);
					*c
				},
				None => {
					//println!("Error {} is not present in content map!!!.", filename);
					[0; MAX_CACHE_FSIZE as usize]
				}
			};
			let index = match self.dq.iter().position(|ref x| x.to_string() == filename) {
				Some(id) => {
					//println!("Index: {}", id);
					id
				}
				None => {
					//println!("Error couldn't find index in DQ{:?}", self.dq);
					0
				}
			};
			//println!("Removing Index: {}", index);
			self.dq.remove(index);
			self.dq.push_back(filename);
			//println!("Returning Content: {}", contents);
			return self.contents;
		} 
		else {
			//Read the file & Update cache
			return self.insert_new_data(filename, mtime);
		}
    }
    
}

impl Worker {

    pub fn work(id: u32, receiver: Arc<Mutex<mpsc::Receiver<TcpStream>>>, base_path: String) {

        let mut core = Core {
            base_path : base_path.clone(),
            max_tick: 1,
            pending: Vec::new(),
			cache: LRUCache::new(CACHE_SIZE),
        };

        loop {
                let stream = receiver.lock().unwrap().recv_timeout(Duration::from_millis(1));
                if !stream.is_ok() {
                    core.tick();
                    continue;
                }
                let mut stream = stream.unwrap();
                //TODO: Parse headers and read files and write to file

                let mut buffer = [0; 512];

                let size = match stream.read(&mut buffer) {
                			Ok(s) => s,
                			Err(_) => {
                				println!();
                				0
                				}
                };
                if(size <= 0) {
                	println!("SRIRAM Empty Header...");
                	let mut response = String::new();
			        response.push_str("HTTP/1.1 404 Not Found\r\n");
			        response.push_str("Content-Type: application/octet-stream\r\n");			        
			        stream.write(response.as_bytes()).unwrap();
                	stream.flush().unwrap(); 
                	continue;
                }
                let buffer = buffer.to_vec();
                let header = String::from_utf8(buffer).unwrap();
                let mut splits = header.split_whitespace();

                splits.next();

                let mut filename = splits.next().unwrap();
                println!("Parsing file:{}", filename);
				if filename == "/" {
					filename = "/index.html"
				}
                let full_filename = format!("{}{}", base_path, filename);
                let file = match File::open(&full_filename) {
                    Ok(file) => file,
                    Err(_) => {
                    	let mut response = String::new();
				        response.push_str("HTTP/1.1 404 Not Found\r\n");
				        response.push_str("Content-Type: application/octet-stream\r\n");
				        response.push_str(&format!("Content-Disposition: inline; filename=\"{}\"\r\n\r\n", filename)[..]);				        
				        stream.write(response.as_bytes()).unwrap();
                    	stream.flush().unwrap(); 
                    	continue
                    },
                };

                //let mut file_buffer:Vec<u8> = Vec::new();
                //file.read_to_end(&mut file_buffer).unwrap();
                //println!("Read from file:{}  len:{}", &full_filename[..], file_buffer.len());

                let mut response = String::new();
                response.push_str("HTTP/1.1 200 OK\r\n");
                response.push_str("Content-Type: application/octet-stream\r\n");
                response.push_str(&format!("Content-Disposition: inline; filename=\"{}\"\r\n\r\n", filename)[..]);
                
                stream.write(response.as_bytes()).unwrap();
                //stream.write(&file_buffer[..]).unwrap();
                stream.flush().unwrap();
                let request:Request = Request{
                    file: file,
                    read: 0,
                    done: false,
                    stream: stream,
                    len: 0,
                    filename: String::from(filename),
                };
                core.add(request);

                core.tick();

            }
    }

    pub fn new(id: u32, base_path: String) -> Worker { //, sender: TcpStream, receiver: TcpStream) -> Worker {
        let (producer, consumer): (Sender<TcpStream>, Receiver<TcpStream>) = mpsc::channel();

        let producer = Arc::new(Mutex::new(producer));
        let consumer = Arc::new(Mutex::new(consumer));
        let cache = LRUCache::new(CACHE_SIZE);
        let worker = Worker {
            id,
            producer,
            consumer,
            cache,
        };        
        worker
    }

    
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Needs 3 arguments: <port> <src-dir> <num-reactors>");
        std::process::exit(-1);
    }
    let port = args[1].parse::<u32>().unwrap();
    let root_dir = &args[2];
    let mut max_cores: u32 = args[3].parse().unwrap();

    let address = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&address).expect("Failed to bind to port");

    println!("Listening on port:{}", port);
    println!("Serving files from: {}", root_dir);
	max_cores = num_cpus::get() as u32;
	println!("Cores available: {}", max_cores);
    let mut workers: Vec<ProducerWorker> = Vec::new();

    for id in 0..max_cores {
        let worker = Worker::new(id, root_dir.clone());
        let consumer = worker.consumer;
        let base_path = root_dir.clone();
        thread::spawn(move || {
            Worker::work(id, consumer, base_path);
        });

        workers.push(ProducerWorker{id: id, producer: worker.producer});
    }

    let mut cur:u32 = 0;
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let producer = &workers[cur as usize].producer;

        producer.lock().unwrap().send(stream).unwrap();
        cur = (cur + 1) % max_cores;
    }
}
