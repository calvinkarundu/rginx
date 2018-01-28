#!/usr/bin/env bash

benchmark() {
  local MAX_REQUESTS=$1
  local MAX_CONCURRENCY=$2
  local i=$3
  local FILENAME=$4
  local SLEEP=$5
  local SERVER=$6
  local IP=$7
  
  echo "Starting AB tests $MAX_REQUESTS $MAX_CONCURRENCY $i $FILENAME $SLEEP $SERVER $IP" 
  mkdir -p results/${SERVER}

  ab -n ${MAX_REQUESTS} -c ${MAX_CONCURRENCY} http://${IP}:8080/jpg/${FILENAME}.jpg > results/${SERVER}/jpg-${i}-${MAX_REQUESTS}-${MAX_CONCURRENCY}-${FILENAME}.txt
  sleep $SLEEP 	
  
echo "Done with first request"
  ab -n ${MAX_REQUESTS} -c ${MAX_CONCURRENCY} http://${IP}:8080/png/${FILENAME}.png > results/${SERVER}/png-${i}-${MAX_REQUESTS}-${MAX_CONCURRENCY}-${FILENAME}.txt
  sleep $SLEEP
  
  ab -n ${MAX_REQUESTS} -c ${MAX_CONCURRENCY} http://${IP}:8080/gif/${FILENAME}.gif > results/${SERVER}/gif-${i}-${MAX_REQUESTS}-${MAX_CONCURRENCY}-${FILENAME}.txt
  sleep $SLEEP

  ab -n ${MAX_REQUESTS} -c ${MAX_CONCURRENCY} http://${IP}:8080/svg/${FILENAME}.svg > results/${SERVER}/svg-${i}-${MAX_REQUESTS}-${MAX_CONCURRENCY}-${FILENAME}.txt
  sleep $SLEEP

  ab -n ${MAX_REQUESTS} -c ${MAX_CONCURRENCY} http://${IP}:8080/css/${FILENAME}.css > results/${SERVER}/css-${i}-${MAX_REQUESTS}-${MAX_CONCURRENCY}-${FILENAME}.txt
  sleep $SLEEP

  ab -n ${MAX_REQUESTS} -c ${MAX_CONCURRENCY} http://${IP}:8080/html/${FILENAME}.html > results/${SERVER}/html-${i}-${MAX_REQUESTS}-${MAX_CONCURRENCY}-${FILENAME}.txt
  sleep $SLEEP

  ab -n ${MAX_REQUESTS} -c ${MAX_CONCURRENCY} http://${IP}:8080/js/${FILENAME}.js > results/${SERVER}/js-${i}-${MAX_REQUESTS}-${MAX_CONCURRENCY}-${FILENAME}.txt
  sleep $SLEEP
}

if [ "$#" -ne 3 ]; then
    echo "Required <IP> <filename-prefix> <server>"
    exit 1
fi

IP=$1
FILENAME=$2
SERVER=$3

mkdir -p results

SLEEP=90
MAX_REQUESTS=1000 
MAX_CONCURRENCY=("10" "100")

for((i=0; i<3; i++)); do
  echo "Starting iteration: ${i}"
  for((j=0; j<2; j++)); do
    benchmark $MAX_REQUESTS ${MAX_CONCURRENCY[$j]} $i $FILENAME $SLEEP $SERVER $IP
  done
done


MAX_REQUESTS=10000 
MAX_CONCURRENCY=("100" "1000")

for((i=0; i<3; i++)); do
  echo "Starting iteration: ${i}"
  for((j=0; j<2; j++)); do
    benchmark $MAX_REQUESTS ${MAX_CONCURRENCY[$j]} $i $FILENAME $SLEEP $SERVER $IP
  done
done


MAX_REQUESTS=100000
MAX_CONCURRENCY=("1000" "5000")

for((i=0; i<3; i++)); do
  echo "Starting iteration: ${i}"
  for((j=0; j<2; j++)); do
    benchmark $MAX_REQUESTS ${MAX_CONCURRENCY[$j]} $i $FILENAME $SLEEP $SERVER $IP
  done
done

echo "Done!"

