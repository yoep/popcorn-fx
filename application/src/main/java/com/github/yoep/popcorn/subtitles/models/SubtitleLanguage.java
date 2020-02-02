package com.github.yoep.popcorn.subtitles.models;

import lombok.Getter;

import java.util.Arrays;

@Getter
public enum SubtitleLanguage {
    NONE("none", "Disabled"),
    ARABIC("ar", "العربية"),
    BULGARIAN("bg", "Български"),
    BOSNIAN("bs", "Bosanski jezik"),
    CZECH("cs", "Český"),
    DANISH("da", "Dansk"),
    GERMAN("de", "Deutsch"),
    MODERN_GREEK("el", "Ελληνικά"),
    ENGLISH("en", "English"),
    SPANISH("es", "Español"),
    ESTONIAN("et", "Eesti"),
    BASQUE("eu", "Euskara"),
    PERSIAN("fa", "فارسی"),
    FINNISH("fi", "Suomi"),
    FRENCH("fr", "Français"),
    HEBREW("he", "עברית"),
    CROATIAN("hr", "Hrvatski"),
    HUNGARIAN("hu", "Magyar"),
    INDONESIAN("id", "Bahasa Indonesia"),
    ITALIAN("it", "Italiano"),
    LITHUANIAN("lt", "lietuvių kalba"),
    DUTCH("nl", "Nederlands"),
    NORWEGIAN("no", "Norsk"),
    POLISH("pl", "Polski"),
    PORTUGUESE("pt", "Português"),
    PORTUGUESE_BRAZIL("pt-br", "Português (Brasil)"),
    ROMANIAN("ro", "română"),
    RUSSIAN("ru", "русский язык"),
    SLOVENE("sl", "slovenščina"),
    SERBIAN("sr", "српски језик"),
    SWEDISH("sv", "svenska"),
    THAI("th", "ไทย"),
    TURKISH("tr", "Türkçe"),
    UKRAINIAN("uk", "українська"),
    VIETNAMESE("vi", "Tiếng Việt");

    private final String code;
    private final String nativeName;

    SubtitleLanguage(String code, String nativeName) {
        this.code = code;
        this.nativeName = nativeName;
    }

    /**
     * Get the language for the given language code.
     *
     * @param code The code of the language to retrieve.
     * @return Returns the language if found, else null.
     */
    public static SubtitleLanguage valueOfCode(String code) {
        return Arrays.stream(values())
                .filter(e -> e.getCode().equalsIgnoreCase(code))
                .findFirst()
                .orElse(null);
    }

    @Override
    public String toString() {
        return nativeName;
    }
}
