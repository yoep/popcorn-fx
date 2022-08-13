package com.github.yoep.popcorn.backend.subtitles.parser;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import lombok.extern.slf4j.Slf4j;

import java.io.InputStream;
import java.nio.charset.Charset;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

//TODO: remove
@Slf4j
public class SrtParser implements Parser {
    @Override
    public boolean support(SubtitleType type) {
        return type == SubtitleType.SRT;
    }

    @Override
    public List<SubtitleCue> parse(InputStream inputStream, Charset encoding) {
        Objects.requireNonNull(inputStream, "inputStream cannot be null");
        Objects.requireNonNull(encoding, "encoding cannot be null");
        return Collections.emptyList();
    }

    @Override
    public InputStream parse(List<SubtitleCue> cues) {
        return null;
    }
}
