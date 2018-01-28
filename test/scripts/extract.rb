#!/usr/bin/env ruby

if ARGV.length == 0 || !['rginx', 'httpd', 'nginx'].include?(ARGV[0])
  puts "Frist argument should be either rginx, httpd, nginx followed by list of *.txt files"
  exit -1
end

server = ARGV[0]
file = File.open("cleandf-#{server}.csv", 'w')
file.puts "server;it;type;rps;tpr;median_tpr;num_conns;concurrency" #Server | It | Type | Requests Per Second | Time Per Request | Median Time Per Request
ARGV[1..ARGV.length].each do |arg|
    filetype = File.basename(arg).split('-')[0]
    it       = arg.split('-')[1]
    rps = 0
    tpr = 0
    median_tpr = 0
    num_conns = arg.split('-')[2]
    concurrency = arg.split('-')[3]

    File.readlines(arg).each do |line|
        line.strip!
        if line.match /^Requests per second/
            rps = line.split(':')[1].strip.split(/\s+/)[0]
        elsif line.match /^Time per request/    
            tpr = line.split(':')[1].strip.split(/\s+/)[0]
        elsif line.match /^50%/    
            median_tpr = line.split(/\s+/)[1]
        end
    end

    file.puts "#{server};#{it};#{filetype};#{rps};#{tpr};#{median_tpr};#{num_conns};#{concurrency}"
end

