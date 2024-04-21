package com.github.yoep.popcorn.ui.stage;

import com.github.yoep.popcorn.ui.view.BorderlessStageWrapper;
import lombok.Data;
import lombok.Setter;

import java.util.Optional;

@Data
public class BorderlessStageHolder {
    private static final BorderlessStageHolder holder = new BorderlessStageHolder();

    @Setter
    private BorderlessStageWrapper stageWrapper;

    private BorderlessStageHolder() {
    }

    public static Optional<BorderlessStageWrapper> getWrapper() {
        return Optional.ofNullable(holder.getStageWrapper());
    }

    public static void setWrapper(BorderlessStageWrapper wrapper) {
        holder.setStageWrapper(wrapper);
    }
}
