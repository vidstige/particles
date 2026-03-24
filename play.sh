#!/bin/sh
RESOLUTION=${RESOLUTION:-720x720}

if [ $# -eq 0 ]; then
    exec ffplay -v warning -f rawvideo -pixel_format rgb32 -framerate 30 -video_size "$RESOLUTION" -i -
fi

AUDIO=$1

ffmpeg -v warning \
  -f rawvideo -pixel_format rgb32 -framerate 30 -video_size "$RESOLUTION" -i pipe:0 \
  -i "$AUDIO" -shortest \
  -c:v rawvideo -pix_fmt rgb32 -c:a copy \
  -f nut - \
| ffplay -v warning -autoexit -
