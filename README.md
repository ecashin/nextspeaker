# NEXTSPEAKER

NEXTSPEAKER is a speaker chooser,
selecting a speaker from a list of participants.

## Goals

* Be unpredictable
* Avoid choosing too-recently chosen speakers
* Avoid neglecting to choose participants for too long

## Usage

Create two text files.

* A list of participants, one name per line, without duplicates
* A list of previous selections, one name per line

## Example

There are two text files.

    bash$ wc -l *.txt
    9 history.txt
    21 participants.txt
    30 total
    bash$ sed 3q participants.txt 
    Alice
    Abram
    Adam
    bash$ sed 3q history.txt 
    Edith
    Earnie
    Estelle

The next speaker is selected.

    bash$ cargo run -- \
        --participants participants.txt \
        --history history.txt
    Abram
    bash$ 

We record the selection in the history.

    bash$ echo Abram >> history.txt
