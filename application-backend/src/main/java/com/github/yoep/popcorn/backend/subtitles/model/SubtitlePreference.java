package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import lombok.Builder;

import java.util.Objects;
import java.util.Optional;

@Builder
public record SubtitlePreference(SubtitlePreferenceTag tag, SubtitleLanguage language) {
    public Optional<SubtitleLanguage> getLanguage() {
        if (tag == SubtitlePreferenceTag.LANGUAGE) {
            return Optional.of(language);
        }

        return Optional.empty();
    }

    public static SubtitlePreference from(com.github.yoep.popcorn.backend.subtitles.ffi.SubtitlePreference preference) {
        Objects.requireNonNull(preference, "preference cannot be null");
        return SubtitlePreference.builder()
            .tag(preference.tag)
            .language(preference.union.language_body.language)
            .build();
    }
}
