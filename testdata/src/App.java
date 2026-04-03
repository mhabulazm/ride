package com.example;

import java.util.List;

public class App {
    private String name;
    private int count = 42;

    public App(String name) {
        this.name = name;
    }

    public void greet() {
        // Print greeting
        System.out.println("Hello, " + name + "!");
        for (int i = 0; i < count; i++) {
            System.out.println("Iteration: " + i);
        }
    }

    public static void main(String[] args) {
        App app = new App("World");
        app.greet();
    }
}
