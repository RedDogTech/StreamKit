# StreamKit
Expierment Rust media sever.

### Dev Setup
Please install the latest version of rust. (nightly not needed).

To install all the deps from cargo run `cargo install` once cloned and cd'ed into.

to pass arguments such as enabling debugging use the `--` such as:

`cargo run -- --log-level debug`

for a list of option and arguments that can be set and what the options mean.

`cargo run -- --help`

#### libSRT
LibSrt is neededs to be installed and in the path brew can be used to fetch the srt libs on MacOS. 

`brew install srt`

### HLS output
The server will send out a fmp4 HLS stream. This can be accessed via.
The streamid is set by the incoming srt stream.

`http://127.0.0.1:3000/{streamid}/playlist.m3u8`

### Example SRT Stream
ffmpeg can be used to send a stream into StreamKit
```
ffmpeg -re \
  -f lavfi -i testsrc=1280x720:r=30000/1001 \
  -f lavfi -i sine=frequency=1000 \
  -vf "settb=AVTB,setpts='trunc(PTS/1K)*1K+st(1,trunc(RTCTIME/1K))-1K*trunc(ld(1)/1K)',drawtext=fontsize=60:fontcolor=black:text='%{localtime}.%{eif\:1M*t-1K*trunc(t*1K)\:d\:3}'" \
  -c:v libx264 -tune zerolatency -b:v 2M -preset ultrafast -r 30 -g 15 -pix_fmt yuv420p \
  -c:a aac -ac 1 -ar 48000 \
  -f mpegts "srt://127.0.0.1:9000?pkt_size=1316&streamid=test"
```

OBS can also be used setting a custom output and using a similar url to:
`srt://127.0.0.1:9000?pkt_size=1316&streamid=test`