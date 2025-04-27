package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.ArrayList;
import java.util.List;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class SubtitleHelper {
    public static String getCode(Subtitle.Language language) {
        return switch (language) {
            case NONE -> "none";
            case CUSTOM -> "custom";
            case ARABIC -> "ar";
            case BULGARIAN -> "bg";
            case BOSNIAN -> "bs";
            case CZECH -> "cs";
            case DANISH -> "da";
            case GERMAN -> "de";
            case MODERN_GREEK -> "el";
            case ENGLISH -> "en";
            case SPANISH -> "es";
            case ESTONIAN -> "et";
            case BASQUE -> "eu";
            case PERSIAN -> "fa";
            case FINNISH -> "fi";
            case FRENCH -> "fr";
            case HEBREW -> "he";
            case CROATIAN -> "hr";
            case HUNGARIAN -> "hu";
            case INDONESIAN -> "id";
            case ITALIAN -> "it";
            case LITHUANIAN -> "lt";
            case DUTCH -> "nl";
            case NORWEGIAN -> "no";
            case POLISH -> "pl";
            case PORTUGUESE -> "pt";
            case PORTUGUESE_BRAZIL -> "pt-br";
            case ROMANIAN -> "ro";
            case RUSSIAN -> "ru";
            case SLOVENE -> "sl";
            case SERBIAN -> "sr";
            case SWEDISH -> "sv";
            case THAI -> "th";
            case TURKISH -> "tr";
            case UKRAINIAN -> "uk";
            case VIETNAMESE -> "vi";
            default -> throw new IllegalArgumentException("Unknown language: " + language);
        };
    }

    public static String getNativeName(Subtitle.Language language) {
        return switch (language) {
            case NONE -> "Disabled";
            case CUSTOM -> "Custom";
            case ARABIC -> "العربية";
            case BULGARIAN -> "Български";
            case BOSNIAN -> "Bosanski jezik";
            case CZECH -> "Český";
            case DANISH -> "Dansk";
            case GERMAN -> "Deutsch";
            case MODERN_GREEK -> "Ελληνικά";
            case ENGLISH -> "English";
            case SPANISH -> "Español";
            case ESTONIAN -> "Eesti";
            case BASQUE -> "Euskara";
            case PERSIAN -> "فارسی";
            case FINNISH -> "Suomi";
            case FRENCH -> "Français";
            case HEBREW -> "עברית";
            case CROATIAN -> "Hrvatski";
            case HUNGARIAN -> "Magyar";
            case INDONESIAN -> "Bahasa Indonesia";
            case ITALIAN -> "Italiano";
            case LITHUANIAN -> "lietuvių kalba";
            case DUTCH -> "Nederlands";
            case NORWEGIAN -> "Norsk";
            case POLISH -> "Polski";
            case PORTUGUESE -> "Português";
            case PORTUGUESE_BRAZIL -> "Português (Brasil)";
            case ROMANIAN -> "română";
            case RUSSIAN -> "русский язык";
            case SLOVENE -> "slovenščina";
            case SERBIAN -> "српски језик";
            case SWEDISH -> "svenska";
            case THAI -> "ไทย";
            case TURKISH -> "Türkçe";
            case UKRAINIAN -> "українська";
            case VIETNAMESE -> "Tiếng Việt";
            default -> throw new IllegalArgumentException("Unknown language: " + language);
        };
    }

    /**
     * Get the flag resource for this subtitle.
     * The flag resource should exist as the "unknown"/"not supported" languages are already filtered by the {@link SubtitleLanguage}.
     *
     * @return Returns the flag class path resource.
     */
    public static String getFlagResource(Subtitle.Language language) {
        return "/images/flags/" + getCode(language) + ".png";
    }

    public static List<Integer> supportedFontSizes() {
        var sizes = new ArrayList<Integer>();

        // increase sizes always by 2
        for (int i = 20; i <= 80; i += 2) {
            sizes.add(i);
        }

        return sizes;
    }
}
