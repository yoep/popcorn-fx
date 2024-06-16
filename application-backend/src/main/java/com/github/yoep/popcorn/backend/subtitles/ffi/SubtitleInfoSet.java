package com.github.yoep.popcorn.backend.subtitles.ffi;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.util.*;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"subtitles", "len"})
public class SubtitleInfoSet extends Structure implements Closeable {
    public static class ByValue extends SubtitleInfoSet implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(List<SubtitleInfo> subtitles) {
            super(subtitles);
        }
    }

    public static class ByReference extends SubtitleInfoSet implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(List<SubtitleInfo> subtitles) {
            super(subtitles);
        }
    }

    public SubtitleInfo.ByReference subtitles;
    public int len;

    private List<SubtitleInfo> cachedSubtitles;

    public SubtitleInfoSet() {
    }

    public SubtitleInfoSet(List<SubtitleInfo> subtitles) {
        Objects.requireNonNull(subtitles, "subtitles not null");
        this.subtitles = new SubtitleInfo.ByReference();
        this.len = subtitles.size();
        this.cachedSubtitles = subtitles;
        var array = (SubtitleInfo[]) this.subtitles.toArray(this.len);

        for (int i = 0; i < this.len; i++) {
            var subtitle = subtitles.get(i);
            array[i].imdbId = subtitle.getImdbId();
            array[i].language = subtitle.getLanguage();
            array[i].files = new SubtitleFile.ByReference();
            array[i].len = subtitle.getLen();

            var fileArray = (SubtitleFile[]) array[i].files.toArray(subtitle.getLen());
            for (int j = 0; j < subtitle.getLen(); j++) {
                var file = subtitle.getFiles().get(j);
                fileArray[j].fileId = file.getFileId();
                fileArray[j].name = file.getName();
                fileArray[j].url = file.getUrl();
                fileArray[j].score = file.getScore();
                fileArray[j].downloads = file.getDownloads();
                fileArray[j].quality = file.getQuality();
                fileArray[j].write();
            }

            array[i].write();
        }

        write();
        setAutoSynch(false);
    }

    public List<SubtitleInfo> getSubtitles() {
        return Optional.ofNullable(cachedSubtitles)
                .orElse(Collections.emptyList());
    }

    @Override
    public void read() {
        super.read();
        cachedSubtitles = Optional.ofNullable(this.subtitles)
                .map(e -> (SubtitleInfo[]) e.toArray(this.len))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() {
        setAutoSynch(false);
        cachedSubtitles.forEach(SubtitleInfo::close);
    }
}
