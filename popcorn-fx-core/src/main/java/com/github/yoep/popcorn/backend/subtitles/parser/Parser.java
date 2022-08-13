package com.github.yoep.popcorn.backend.subtitles.parser;

import com.github.yoep.popcorn.backend.subtitles.SubtitleParsingException;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;

import java.io.InputStream;
import java.nio.charset.Charset;
import java.util.List;

public interface Parser {
    /**
     * Check if this parser support the given subtitle type.
     *
     * @param type The type of the subtitle file to support.
     * @return Returns true if the parser support the format, else false.
     */
    boolean support(SubtitleType type);

    /**
     * Parse the given input to subtitle cues.
     *
     * @param inputStream The input to parse.
     * @param encoding    The encoding of the input.
     * @return Returns the subtitles cues of the given input.
     * @throws SubtitleParsingException Is thrown when an error occurs during parsing of the input.
     */
    List<SubtitleCue> parse(InputStream inputStream, Charset encoding);

    /**
     * Parse the given subtitle cues to the subtitle format.
     * The {@link InputStream} is always encoded as {@link java.nio.charset.StandardCharsets#UTF_8}.
     * The {@link InputStream} contains the same value as was given to the {@link #parse(InputStream, Charset)} method.
     *
     * @param cues The subtitle cues to parse.
     * @return Returns the input stream of the cues as the subtitle format.
     */
    InputStream parse(List<SubtitleCue> cues);
}
