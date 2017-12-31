#!/bin/bash
# This shell script generates screenshots from the relm examples. And generates
# the README.md of the examples dir. At the moment the Cargo sub projects
# (e.g. webkit-test) into the examples dir are'nt supported.
#
# **This script was only tested under linux!**
#
# ## Dependencies
# You need the following dependencies localy installed,
# beside the obvious relm dependencies:
# - imagemagick
# - xvfb
# - psmisc (for killall)

OUTDIR=examples
MARKDOWN=${OUTDIR}/README.md

cleanup() {
  [ -f ${MARKDOWN} ] && rm ${MARKDOWN}
}

screenshot() {
  EXAMPLE=${1}
  # start Xvfb virtual Xserver
  Xvfb :99 -ac -screen 0 400x300x16 &>/dev/null &
  # build example
  cargo build --example ${EXAMPLE}
  # run example
  env DISPLAY=:99 cargo run --example ${EXAMPLE} &>/dev/null &
  # give example time to start
  sleep 2
  # create screenshot
  env DISPLAY=:99 import -window root ${OUTDIR}/${EXAMPLE}.png &>/dev/null
  # crop image
  convert ${OUTDIR}/${EXAMPLE}.png -trim ${OUTDIR}/${EXAMPLE}.png

  # stop virtual Xserver
  killall Xvfb &>/dev/null
}

# Header
tableheader() {
  echo -e "# Relm Example Screenshots\n" >> ${MARKDOWN}
}

# for each example one row
tablerow() {
  EXAMPLE=${1}

  echo -e "## ${EXAMPLE}\n" >> ${MARKDOWN}
  echo -e "[https://github.com/antoyo/relm/tree/master/examples/${EXAMPLE}.rs](https://github.com/antoyo/relm/tree/master/examples/${EXAMPLE}.rs)\n" >> ${MARKDOWN}
  echo -e "![Example: ${EXAMPLE}](${EXAMPLE}.png)\n" >> ${MARKDOWN}
}

tablefooter() {
  echo -e "----\n" >> ${MARKDOWN}
  echo -e "More info here: [https://github.com/antoyo/relm](https://github.com/antoyo/relm)\n" >> ${MARKDOWN}
}

# START
cleanup

tableheader

for example in $(find examples/ -maxdepth 1 -name "*.rs" -type f); do
  EXAMPLE=$(basename ${example} .rs)

  screenshot ${EXAMPLE}
  tablerow ${EXAMPLE}
done

tablefooter
