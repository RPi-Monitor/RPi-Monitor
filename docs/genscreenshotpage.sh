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
        echo >> ${TARGET}
        echo "-----" >> ${TARGET}
        echo >> ${TARGET}
        echo "<figure>" >> ${TARGET}
        if [ "" !=  "${WIDTH}" ]
        then
            echo "<img src=\"${IMAGE}\" width=\"${WIDTH%px}\">" >> ${TARGET}
        else
            echo "<img src=\"${IMAGE}\">" >> ${TARGET}
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