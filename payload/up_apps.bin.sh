

if [ -z "$1" ]; then
    printf "Usage: at least one [userapp bin path]\n"
    exit
fi


dd if=/dev/zero of=./apps.bin bs=1M count=32
  
BASE=0

add_to_img()
{
  printf "%08x" 0xCAFEBABE | xxd -r -ps > /tmp/head.bin
  # stat -c "%s" $1  2>/dev/null || 
  SIZE1=`stat -f "%z" $1`
  echo "file $1 , size : $SIZE1 , current offset : $BASE"
  printf "%08x" $SIZE1 | xxd -r -ps >> /tmp/head.bin
  dd if=/tmp/head.bin of=./apps.bin seek=$BASE obs=1  conv=notrunc
  BASE=$((BASE + 8))
  dd if=$1 of=./apps.bin seek=$BASE obs=1 conv=notrunc
  BASE=$((BASE + SIZE1))
}

add_to_img $1
if [  "$2" ]; then
add_to_img $2
fi
# xxd -l 16 ./payload/apps.bin                 
# 00000000: cafe babe 0000 0006 7300 5010 0000 0000  ........s.P.....
