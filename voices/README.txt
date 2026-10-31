Spoken notices of EH3D-BUNGARD-GUI (and of the screen EH3D-NODE-BUNGARD-HMI)

What this folder is
-------------------
Each notice has four sentences: one when the job ends, one when the machine asks for a tool, and two for the visual check
of the board with the camera (correct and not correct). They are in the folders LANGUAGE/GENDER/finished.wav,
LANGUAGE/GENDER/tool.wav, LANGUAGE/GENDER/check_ok.wav and LANGUAGE/GENDER/check_bad.wav, where LANGUAGE is
en, es, it, de, fr, pt, zh, ja or one of the other codes listed below and GENDER is f (female voice) or m (male voice). They are chosen in the list
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

Voices of the other languages (all free to give away; the first three need the attribution in their line):

bg/m      bg_BG-dimitar-medium              CC0
cs/m      cs_CZ-jirka-medium                CC0
da/m      da_DK-talesyntese-medium          CC0
nl/m      nl_NL-alex-medium                 CC0
et/f      et_EE-news-medium (speaker Mari)  CC BY (attribution: Speech corpus of Estonian news sentences, University of Tartu / Institute of the Estonian Language)
fi/m      fi_FI-harri-medium                CC0
el/f      el_GR-rapunzelina-medium          CC0
hu/f      hu_HU-anna-medium                 CC0
lv/m      lv_LV-aivars-medium               CC0
no/m      no_NO-talesyntese-medium          CC0
pl/m      pl_PL-darkman-medium              CC0
ro/m      ro_RO-mihai-medium                CC0
sk/f      sk_SK-lili-medium                 CC0
sl/m      sl_SI-artur-medium                CC BY 4.0 (attribution: the Artur studio TTS dataset, ppisljar)
sv/m      sv_SE-nst-medium                  CC0
uk/f      uk_UA-ukrainian_tts-medium (speaker tetiana)   CC0
ru/m      ru_RU-dmitri-medium               CC0
sq/m      sq_AL-edon-medium                 CC0
eu/m      eu_ES-antton-medium               CC BY 4.0 (attribution: the Antton dataset, Itzune)
pt-br/m   pt_BR-faber-medium                CC0 (Brazilian Portuguese)

The gender of these voices was not heard by the person who made them: it is the one that the name of the voice says (checked against its pitch, which
did not decide it for Czech, Dutch and Russian). Listen to them and move a folder from m to f, or the other way, if it is wrong.

Languages that have no voice, and why (they play the tone): Croatian, Irish, Maltese and Galician (Piper has no voice for them); Catalan (the
only voices are CC BY-SA, which cannot be given away inside a GPL repository); Serbian (CC BY-NC-SA, not free for this use); Icelandic (the licence of
its data is not stated in the card of the voice); Lithuanian (the one free voice needs a phonemizer that this version of Piper does not have and it was
trained from a voice of unknown licence); Canadian French uses the French voices. The Russian voices Ruslan (NC-SA) and Irina (unknown) were left out;
the Greek voice Joy (NC) too.

Missing, for now, of the first eight languages: Japanese (the only voice for Piper is not free for this use), the Chinese and the Portuguese female
voices (no free voice exists), so these three are to be recorded by a person. The gender of each voice is the one
that the name of the voice and its pitch say; listen to them before you rely on it.

The sentences
-------------
Job ended:  "The job has finished. You may remove the board." (and the same in each language)
Tool:       "Please change the tool."                          (and the same in each language)
Visual check, correct:     "The visual check of the board is correct."     (and the same in each language)
Visual check, not correct: "The visual check of the board is not correct." (and the same in each language)
A voice that has no recording of the check plays the tone instead, so an older folder of voices still works.
