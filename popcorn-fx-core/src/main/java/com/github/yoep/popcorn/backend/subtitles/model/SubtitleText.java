package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.ToString;

import java.io.Closeable;

@Getter
@ToString
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"text", "italic", "bold", "underline"})
public class SubtitleText extends Structure implements Closeable {
    public static class ByReference extends SubtitleText implements Structure.ByReference {
    }

    public String text;
    public byte italic;
    public byte bold;
    public byte underline;

    public boolean isItalic() {
        return italic == 1;
    }

    public boolean isBold() {
        return bold == 1;
    }

    public boolean isUnderline() {
        return underline == 1;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
