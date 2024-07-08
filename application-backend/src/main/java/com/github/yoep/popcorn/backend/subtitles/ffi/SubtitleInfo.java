package com.github.yoep.popcorn.backend.subtitles.ffi;

import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * The subtitle info contains information about available subtitles for a certain IMDB ID.
 * This info includes a specific language for the media ID as well as multiple available files which can be used for smart subtitle detection.
 */
@Slf4j
@Data
@ToString
@EqualsAndHashCode(of = {"imdbId", "language"}, callSuper = false)
@Structure.FieldOrder({"imdbId", "language", "files", "len"})
public class SubtitleInfo extends Structure implements Closeable {
    @EqualsAndHashCode(callSuper = true)
    public static class ByReference extends SubtitleInfo implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(String imdbId, SubtitleLanguage language, SubtitleFile.ByReference... files) {
            super(imdbId, language, files);
        }

        public static SubtitleInfo.ByReference from(com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo info) {
            return new SubtitleInfo.ByReference(info.imdbId(), info.language(), info.files().stream()
                    .map(SubtitleFile.ByReference::from)
                    .toArray(SubtitleFile.ByReference[]::new));
        }
    }

    public String imdbId;
    public SubtitleLanguage language;
    public SubtitleFile.ByReference files;
    public int len;

    //region Constructors

    public SubtitleInfo() {
        super();
    }

    @Builder
    public SubtitleInfo(String imdbId, SubtitleLanguage language, SubtitleFile.ByReference... files) {
        files = files == null ? new SubtitleFile.ByReference[0] : files;
        this.imdbId = imdbId;
        this.language = language;
        this.files = files.length > 0 ? new SubtitleFile.ByReference() : null;
        this.len =  files.length;

        if (this.len > 0) {
            var array = (SubtitleFile.ByReference[]) this.files.toArray(this.len);
            for (int i = 0; i < this.len; i++) {
                var file = files[i];
                array[i].name = file.name;
                array[i].url = file.url;
                array[i].fileId = file.fileId;
                array[i].score = file.score;
                array[i].downloads = file.downloads;
                array[i].quality = file.quality;
            }
            write();
        }
    }

    //endregion

    //region Getters & Setters

    /**
     * Check if this subtitle is the special "none" subtitle.
     *
     * @return Returns true if this subtitle is the "none" subtitle, else false.
     */
    public boolean isNone() {
        return getLanguage() == SubtitleLanguage.NONE;
    }

    /**
     * Check if this subtitle is the special "custom" subtitle.
     *
     * @return Returns true if this subtitle is the "custom" subtitle, else false.
     */
    public boolean isCustom() {
        return getLanguage() == SubtitleLanguage.CUSTOM;
    }

    public List<SubtitleFile> getFiles() {
        return Optional.ofNullable(files)
                .map(e -> (SubtitleFile[]) files.toArray(len))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    //endregion

    //region Methods

    @Override
    public void close() {
        setAutoSynch(false);
        getFiles().forEach(SubtitleFile::close);
    }

    //endregion
}
