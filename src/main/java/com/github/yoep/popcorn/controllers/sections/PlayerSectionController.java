package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.video.VideoPlayer;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.ImageView;
import lombok.Getter;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.ResourceBundle;

@Controller
public class PlayerSectionController implements Initializable {
    @Getter
    private VideoPlayer videoPlayer;

    @FXML
    private ImageView videoView;

    public PlayerSectionController() {

    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeVideoPlayer();
    }

    private void initializeVideoPlayer() {
        if (videoPlayer != null)
            return;

        videoView.setVisible(false);

        this.videoPlayer = new VideoPlayer(videoView);
    }
}
