Spoken notices of EH3D-BUNGARD-GUI (and of the screen EH3D-NODE-BUNGARD-HMI)

What this folder is
-------------------
Each notice has two sentences: one when the job ends and one when the machine asks for a tool.
They are in the folders LANGUAGE/GENDER/finished.wav and LANGUAGE/GENDER/tool.wav, where LANGUAGE is
en, es, it, de, fr, pt, zh or ja and GENDER is f (female voice) or m (male voice). They are chosen in the list
of sounds of the settings of the program (or with `sound_finish = voice:es:f` in the machine file).

IMPORTANT: these are NOT recordings of a person. They are synthetic voices, made with the neural text-to-speech
program Piper (https://github.com/OHF-Voice/piper1-gpl), spoken a little slower than usual and with a formal
sentence, as a starting set. The voice of a person of age and a professional tone cannot be made by a synthesizer:
to have it, record your own notices and put the WAV files in the same folders (16 bits, mono, any rate); they
replace these ones. A language or a gender that has no folder is not offered in the list.

What is here and what is not
----------------------------
Only the voices whose licence allows to give them away are included:

en/m   en_US-joe-medium            CC0
en/f   en_GB-cori-medium           public domain
es/m   es_ES-davefx-medium         CC0
es/f   es_ES-sharvard-medium (speaker 1)   CC BY 3.0 (attribution: Harvard sentences, Spanish)
it/m   it_IT-riccardo-x_low        from the M-AILABS Speech Dataset (see https://www.caito.de/2019/01/03/the-m-ailabs-speech-dataset/ for its terms)
it/f   it_IT-serena-medium         CC BY 4.0 (attribution)
de/m   de_DE-thorsten-medium       CC0
de/f   de_DE-kerstin-low           CC0
fr/m   fr_FR-gilles-low            CC0
fr/f   fr_FR-siwis-medium          CC BY 4.0 (attribution)
pt/m   pt_PT-tugao-medium          CC0 (European Portuguese)
zh/m   zh_CN-chaowen-medium        CC0

Missing, for now: Japanese (the only voice for Piper is not free for this use), the Chinese and the Portuguese female
voices (no free voice exists), so these three are to be recorded by a person. The gender of each voice is the one
that the name of the voice and its pitch say; listen to them before you rely on it.

The sentences
-------------
Job ended:  "The job has finished. You may remove the board." (and the same in each language)
Tool:       "Please change the tool."                          (and the same in each language)
