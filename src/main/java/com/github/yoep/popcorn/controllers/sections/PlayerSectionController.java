package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.activities.ActivityManager;
import javafx.fxml.Initializable;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.ResourceBundle;

@Controller
@RequiredArgsConstructor
public class PlayerSectionController implements Initializable {
    private final ActivityManager activityManager;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
    }
}
