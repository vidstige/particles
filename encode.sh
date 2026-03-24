#!/bin/sh
RESOLUTION=${RESOLUTION:-720x720}
FPS=${FPS:-30}
if [[ $1 == *.gif ]]; then
    VF=-vf "fps=$FPS,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse"
else
    VF=-"c:v libx264 -crf 20 -preset slow -vf format=yuv420p"
fi

ffmpeg -hide_banner -f rawvideo -pixel_format rgb32 -r $FPS  -framerate $FPS -video_size $RESOLUTION -i - $VF $1
