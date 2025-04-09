package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import java.io.InputStream;
import java.util.Optional;

@Slf4j
public class SubtitleDropDownButton extends DropDownButton<Subtitle.Info> {
    public SubtitleDropDownButton() {
        super(createItemFactory());
    }

    private static Image resourceToImage(InputStream resource) {
        return new Image(resource, 16, 16, true, true);
    }

    private static DropDownButtonFactory<Subtitle.Info> createItemFactory() {
        return new DropDownButtonFactory<>() {
            @Override
            public String getId(Subtitle.Info item) {
                return Optional.ofNullable(item.getImdbId())
                        .orElseGet(() -> SubtitleHelper.getCode(item.getLanguage()));
            }

            @Override
            public String displayName(Subtitle.Info item) {
                return SubtitleHelper.getNativeName(item.getLanguage());
            }

            @Override
            public Image graphicResource(Subtitle.Info item) {
                return resourceToImage(SubtitleDropDownButton.class.getResourceAsStream(
                        SubtitleHelper.getFlagResource(item.getLanguage())
                ));
            }
        };
    }
}
