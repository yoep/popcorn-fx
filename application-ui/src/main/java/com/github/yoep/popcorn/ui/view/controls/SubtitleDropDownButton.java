package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import java.io.InputStream;
import java.util.Optional;

@Slf4j
public class SubtitleDropDownButton extends DropDownButton<ISubtitleInfo> {
    public SubtitleDropDownButton() {
        super(createItemFactory());
    }

    private static Image resourceToImage(InputStream resource) {
        return new Image(resource, 16, 16, true, true);
    }

    private static DropDownButtonFactory<ISubtitleInfo> createItemFactory() {
        return new DropDownButtonFactory<>() {
            @Override
            public String getId(ISubtitleInfo item) {
                return Optional.ofNullable(item.getImdbId())
                        .filter(e -> !e.isEmpty())
                        .orElseGet(() -> SubtitleHelper.getCode(item.getLanguage()));
            }

            @Override
            public String displayName(ISubtitleInfo item) {
                return SubtitleHelper.getNativeName(item.getLanguage());
            }

            @Override
            public Image graphicResource(ISubtitleInfo item) {
                return resourceToImage(SubtitleDropDownButton.class.getResourceAsStream(
                        SubtitleHelper.getFlagResource(item.getLanguage())
                ));
            }
        };
    }
}
