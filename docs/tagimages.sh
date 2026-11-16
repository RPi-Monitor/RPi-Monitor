#!/bin/bash
for IMAGE in $(ls user_manual/images/*.png)
do
    qiv ${IMAGE} &
    PID=$!
    
    exiftool ${IMAGE} | perl -ne '/Doc Description\s+:\s(.*)/ and print "$1"'
    echo
    echo -n "Description: "
    read DESCRIPTION
    if [ "" != "${DESCRIPTION}" ]
    then
        WIDTH=$(exiftool ${IMAGE} | perl -ne '/Image Width\s+:\s(\d+)/ and print "$1"')
        if [ WIDTH > 500 ]
        then
            convert ${IMAGE} -set 'Doc Width' '500px' ${IMAGE}
        fi

        convert ${IMAGE} -set 'Doc Description' "${DESCRIPTION}" ${IMAGE}
    fi
    kill -9 ${PID} > /dev/null 2>&1
    echo
done