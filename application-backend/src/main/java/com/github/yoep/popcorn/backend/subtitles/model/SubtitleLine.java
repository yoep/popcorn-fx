package com.github.yoep.popcorn.backend.subtitles.model;

import lombok.Builder;

import java.util.List;

@Builder
public record SubtitleLine(List<SubtitleText> texts) {
}
