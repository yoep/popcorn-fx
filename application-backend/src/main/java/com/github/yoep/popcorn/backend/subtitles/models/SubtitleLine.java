package com.github.yoep.popcorn.backend.subtitles.models;

import lombok.AllArgsConstructor;
import lombok.Data;

import java.util.List;

@Data
@AllArgsConstructor
public class SubtitleLine {
    private List<SubtitleText> texts;
}
