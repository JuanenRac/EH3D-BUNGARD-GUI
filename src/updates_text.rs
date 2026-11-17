//! The two phrases the window of the updates adds to the result of an installation: whether the package carried a signature of the release key. They live
//! here, beside the update, and are written in the 37 languages of the program (a variant uses the phrases of its main language).

use eh3d_view::texts::Lang;

/// "signed by the release key" or "not signed: only the checksum was checked", in the language of the window.
pub fn signature_state(lang: Lang, signed: bool) -> &'static str {
    let (yes, no) = match lang.code() {
        "es" => (
            "firmada con la clave de la versión",
            "sin firma: solo se comprobó la suma",
        ),
        "it" => (
            "firmato con la chiave della versione",
            "senza firma: è stata controllata solo la somma",
        ),
        "de" => (
            "mit dem Schlüssel der Version signiert",
            "nicht signiert: nur die Prüfsumme wurde geprüft",
        ),
        "fr" => (
            "signé avec la clé de la version",
            "non signée : seule la somme a été vérifiée",
        ),
        "pt" => (
            "assinada com a chave da versão",
            "sem assinatura: só se verificou a soma",
        ),
        "zh" => ("已用发布密钥签名", "未签名：仅校验了校验和"),
        "ja" => ("リリース鍵で署名済み", "署名なし：チェックサムのみ確認"),
        "bg" => (
            "подписана с ключа на версията",
            "не е подписана: проверена е само контролната сума",
        ),
        "hr" => (
            "potpisano ključem izdanja",
            "nije potpisano: provjeren je samo kontrolni zbroj",
        ),
        "cs" => (
            "podepsáno klíčem vydání",
            "nepodepsáno: ověřen byl jen kontrolní součet",
        ),
        "da" => (
            "signeret med udgivelsens nøgle",
            "ikke signeret: kun kontrolsummen blev kontrolleret",
        ),
        "nl" => (
            "ondertekend met de sleutel van de release",
            "niet ondertekend: alleen de controlesom is gecontroleerd",
        ),
        "et" => (
            "allkirjastatud väljalaske võtmega",
            "allkirjastamata: kontrolliti ainult kontrollsummat",
        ),
        "fi" => (
            "allekirjoitettu julkaisun avaimella",
            "ei allekirjoitettu: vain tarkistussumma tarkistettiin",
        ),
        "el" => (
            "υπογεγραμμένο με το κλειδί της έκδοσης",
            "χωρίς υπογραφή: ελέγχθηκε μόνο το άθροισμα ελέγχου",
        ),
        "hu" => (
            "a kiadás kulcsával aláírva",
            "nincs aláírva: csak az ellenőrző összeget vizsgáltuk",
        ),
        "ga" => (
            "sínithe le heochair an eisithe",
            "gan síniú: ní dhearnadh ach an tsuim seiceála a sheiceáil",
        ),
        "lv" => (
            "parakstīts ar izlaiduma atslēgu",
            "nav parakstīts: pārbaudīta tikai kontrolsumma",
        ),
        "lt" => (
            "pasirašyta išleidimo raktu",
            "nepasirašyta: patikrinta tik kontrolinė suma",
        ),
        "mt" => (
            "iffirmat bl-ċavetta tal-verżjoni",
            "mhux iffirmat: ġie ċċekkjat biss it-total ta' kontroll",
        ),
        "pl" => (
            "podpisano kluczem wydania",
            "bez podpisu: sprawdzono tylko sumę kontrolną",
        ),
        "ro" => (
            "semnat cu cheia versiunii",
            "nesemnat: s-a verificat doar suma de control",
        ),
        "sk" => (
            "podpísané kľúčom vydania",
            "nepodpísané: overil sa len kontrolný súčet",
        ),
        "sl" => (
            "podpisano s ključem izdaje",
            "ni podpisano: preverjena je bila samo nadzorna vsota",
        ),
        "sv" => (
            "signerad med utgåvans nyckel",
            "inte signerad: bara kontrollsumman kontrollerades",
        ),
        "no" => (
            "signert med utgivelsens nøkkel",
            "ikke signert: bare sjekksummen ble kontrollert",
        ),
        "is" => (
            "undirritað með lykli útgáfunnar",
            "óundirritað: aðeins var athugað hvort samtalan stæðist",
        ),
        "uk" => (
            "підписано ключем випуску",
            "не підписано: перевірено лише контрольну суму",
        ),
        "ru" => (
            "подписано ключом выпуска",
            "не подписано: проверена только контрольная сумма",
        ),
        "sr" => (
            "потписано кључем издања",
            "није потписано: проверен је само контролни збир",
        ),
        "sq" => (
            "e nënshkruar me çelësin e botimit",
            "pa nënshkrim: u kontrollua vetëm shuma e kontrollit",
        ),
        "ca" => (
            "signada amb la clau de la versió",
            "sense signatura: només s'ha comprovat la suma",
        ),
        "gl" => (
            "asinada coa chave da versión",
            "sen sinatura: só se comprobou a suma",
        ),
        "eu" => (
            "bertsioaren giltzarekin sinatua",
            "sinadurarik gabe: batura bakarrik egiaztatu da",
        ),
        "pt-br" => return signature_state(Lang::Pt, signed),
        "fr-ca" => return signature_state(Lang::Fr, signed),
        _ => (
            "signed by the release key",
            "not signed: only the checksum was checked",
        ),
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

    const LANGS: &[&str] = &[
        "en", "es", "it", "de", "fr", "pt", "zh", "ja", "bg", "hr", "cs", "da", "nl", "et", "fi",
        "el", "hu", "ga", "lv", "lt", "mt", "pl", "ro", "sk", "sl", "sv", "no", "is", "uk", "ru",
        "sr", "sq", "ca", "gl", "eu", "pt-br", "fr-ca",
    ];

    #[test]
    fn every_language_says_both_phrases_and_they_differ() {
        for code in LANGS {
            let lang = Lang::from_code(code).unwrap_or_else(|| panic!("{code} is not a language"));
            let (yes, no) = (signature_state(lang, true), signature_state(lang, false));
            assert!(!yes.trim().is_empty() && !no.trim().is_empty(), "{code}");
            assert_ne!(yes, no, "{code}");
        }
    }

    #[test]
    fn the_variants_use_the_phrases_of_their_main_language() {
        let (pt_br, fr_ca) = (Lang::from_code("pt-br"), Lang::from_code("fr-ca"));
        assert_eq!(
            signature_state(pt_br.unwrap(), true),
            signature_state(Lang::Pt, true)
        );
        assert_eq!(
            signature_state(fr_ca.unwrap(), false),
            signature_state(Lang::Fr, false)
        );
    }
}
