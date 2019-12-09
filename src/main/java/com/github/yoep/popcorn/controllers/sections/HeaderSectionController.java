package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.models.Genre;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ComboBox;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;
import java.util.stream.Collectors;

@Controller
@RequiredArgsConstructor
public class HeaderSectionController implements Initializable {
    private final PopcornProperties popcornProperties;
    private final LocaleText localeText;

    @FXML
    private ComboBox<Genre> genreCombo;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeGenres();
    }

    private void initializeGenres() {
        List<Genre> genres = popcornProperties.getGenres().stream()
                .map(e -> new Genre(e, localeText.get("genre_" + e)))
                .sorted()
                .collect(Collectors.toList());

        genreCombo.getItems().addAll(genres);
        genreCombo.getSelectionModel().select(0);
    }

    @FXML
    private void onGenreClicked() {
        genreCombo.show();
    }
}
