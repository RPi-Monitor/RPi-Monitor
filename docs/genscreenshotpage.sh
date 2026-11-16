#!/bin/bash

# Add legend in png image with the following command:
#  convert image.png -set 'Doc Width' '500px' -set 'Doc Description' 'Status page' image.png
TARGET=02_screenshots.md


cd source
echo "# Screenshots" > ${TARGET}

for IMAGE in $(ls _static/*.png)
do
    DESCRIPTION=$(exiftool ${IMAGE} | perl -ne '/Doc Description\s+:\s+(.*)/ and print $1' )
    if [ "" != "${DESCRIPTION}" ]
    then
        WIDTH=$(exiftool ${IMAGE} | perl -ne '/Doc Width\s+:\s+(.*)/ and print $1' )
        # A vector version, when present, is shown instead; the PNG keeps the metadata.
        SOURCE=${IMAGE%.png}.svg
        [ -f "${SOURCE}" ] || SOURCE=${IMAGE}
        echo >> ${TARGET}
        echo "-----" >> ${TARGET}
        echo >> ${TARGET}
        echo "<figure>" >> ${TARGET}
        if [ "" !=  "${WIDTH}" ]
        then
            echo "<img src=\"${SOURCE}\" width=\"${WIDTH%px}\">" >> ${TARGET}
        else
            echo "<img src=\"${SOURCE}\">" >> ${TARGET}
        fi
        # Blank lines around the description let Markdown render inside the caption.
        echo "<figcaption>" >> ${TARGET}
        echo >> ${TARGET}
        echo "${DESCRIPTION}" >> ${TARGET}
        echo >> ${TARGET}
        echo "</figcaption>" >> ${TARGET}
        echo "</figure>" >> ${TARGET}
    fi
done