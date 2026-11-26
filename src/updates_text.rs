//! The two phrases the window of the updates adds to the result of an installation: whether the package carried a signature of the release key. They live
//! here, beside the update, and are written in the eight languages of the program.

use eh3d_view::texts::Lang;

/// "signed by the release key" or "not signed: only the checksum was checked", in the language of the window.
pub fn signature_state(lang: Lang, signed: bool) -> &'static str {
    let (yes, no) = match lang {
        Lang::En => (
            "signed by the release key",
            "not signed: only the checksum was checked",
        ),
        Lang::Es => (
            "firmada con la clave de la versión",
            "sin firma: solo se comprobó la suma",
        ),
        Lang::It => (
            "firmato con la chiave della versione",
            "senza firma: è stata controllata solo la somma",
        ),
        Lang::De => (
            "mit dem Schlüssel der Version signiert",
            "nicht signiert: nur die Prüfsumme wurde geprüft",
        ),
        Lang::Fr => (
            "signé avec la clé de la version",
            "non signée : seule la somme a été vérifiée",
        ),
        Lang::Pt => (
            "assinada com a chave da versão",
            "sem assinatura: só se verificou a soma",
        ),
        Lang::Zh => ("已用发布密钥签名", "未签名：仅校验了校验和"),
        Lang::Ja => ("リリース鍵で署名済み", "署名なし：チェックサムのみ確認"),
    };
    if signed {
        yes
    } else {
        no
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_says_both_phrases_and_they_differ() {
        for lang in Lang::ALL {
            let (yes, no) = (signature_state(lang, true), signature_state(lang, false));
            assert!(
                !yes.trim().is_empty() && !no.trim().is_empty(),
                "{}",
                lang.code()
            );
            assert_ne!(yes, no, "{}", lang.code());
        }
    }
}
