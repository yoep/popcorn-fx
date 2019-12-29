package com.github.yoep.popcorn.subtitle;

import lombok.Getter;

import java.nio.charset.Charset;
import java.util.Arrays;

@Getter
public class Language {
    private static final Language[] VALUES = new Language[]{
            new Language("none", "Disabled"),
            new Language("ar", "العربية"),
            new Language("bg", "Български"),
            new Language("bs", "Bosanski jezik"),
            new Language("cs", "Český"),
            new Language("da", "Dansk"),
            new Language("de", "Deutsch"),
            new Language("el", "Ελληνικά"),
            new Language("en", "English", "iso-8859-1"),
            new Language("es", "Español"),
            new Language("et", "Eesti"),
            new Language("eu", "Euskara"),
            new Language("fa", "فارسی"),
            new Language("fi", "Suomi"),
            new Language("fr", "Français"),
            new Language("he", "עברית"),
            new Language("hr", "Hrvatski"),
            new Language("hu", "Magyar"),
            new Language("id", "Bahasa Indonesia"),
            new Language("it", "Italiano"),
            new Language("lt", "lietuvių kalba"),
            new Language("nl", "Nederlands"),
            new Language("no", "Norsk"),
            new Language("pl", "Polski"),
            new Language("pt", "Português"),
            new Language("pt-br", "Português (Brasil)"),
            new Language("ro", "română"),
            new Language("ru", "русский язык"),
            new Language("sl", "slovenščina"),
            new Language("sr", "српски језик"),
            new Language("sv", "svenska"),
            new Language("th", "ไทย"),
            new Language("tr", "Türkçe"),
            new Language("uk", "українська"),
            new Language("vi", "Tiếng Việt")
    };

    private final String code;
    private final String nativeName;
    private final Charset encoding;

    private Language(String code, String nativeName) {
        this.code = code;
        this.nativeName = nativeName;
        this.encoding = Charset.defaultCharset();
    }

    public Language(String code, String nativeName, String encoding) {
        this.code = code;
        this.nativeName = nativeName;
        this.encoding = Charset.forName(encoding);
    }

    public static Language valueOf(String code) {
        return Arrays.stream(VALUES)
                .filter(e -> e.getCode().equalsIgnoreCase(code))
                .findFirst()
                .orElse(null);
    }
}
