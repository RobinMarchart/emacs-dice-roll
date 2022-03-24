;;; dice-roll.el --- indulge in your crippling dice addiction -*- lexical-binding: t; -*-
;;
;; Copyright (C) 2022 Robin Marchart
;;
;; Author: Robin Marchart
;; Maintainer: Robin Marchart
;; Created: März 24, 2022
;; Modified: März 24, 2022
;; Version: 0.0.1
;; Homepage: https://github.com/robin/dice-roll
;; Package-Requires: ((emacs "26.1")(native-async-rs "0.2"))
;;
;; This file is not part of GNU Emacs.
;;
;;; Commentary:
;;
;;  endulge in your crippeling dice addiction
;;
;;; Code:

(defvar dice-roll-build-silent nil "Don't ask if library should be build if t.")

(defvar dice-roll--rootdir (expand-file-name(file-name-directory (or load-file-name buffer-file-name)))"Local Directory of dice-roll repo.")
(defvar dice-roll--module-path (expand-file-name (concat "target/release/libnative_setup" module-file-suffix) dice-roll--rootdir) "Path of the native module.")
(defvar dice-roll--compile-command '("cargo" "build" "--release") "Command to compile native module.")

(defvar dice-roll--ensure-native-promise nil "Cached promise for ensure-native.")

(add-variable-watcher 'dice-roll--rootdir
                      (lambda (_symbol value op _where)
                        (when (eq op 'set)
                          (set 'dice-roll--module-path (expand-file-name (concat "target/release/libnative_setup" module-file-suffix) value)))))

(defun dice-roll--setup-function (resolve reject)
  "Build the native module.
Calls RESOLVE with nil on success, REJECT on failure"
  (if (and (require 'dice-roll-impl dice-roll--module-path t) (file-executable-p dice-roll--executable-path))
      (funcall resolve ())
    (if (or dice-roll-build-silent (y-or-n-p "Dice-roll needs to be build. do it now?"))
        (let (
              (buffer (get-buffer-create "dice-roll-build"))
              (default-directory (file-name-as-directory dice-roll--rootdir))
              (process-connection-type nil))
          (with-current-buffer buffer
            (compilation-mode)
            (setq-local
             default-directory (file-name-as-directory dice-roll--rootdir)
             process-connection-type nil)
            (set-process-sentinel
             (apply #'start-process "dice-roll-build" buffer dice-roll--compile-command)
             (lambda (_process event)
               (pcase event
                 ("finished\n" (require 'dice-roll-impl) (funcall resolve nil))
                 ((rx (| (seq "open" (* anychar)) "run\n")))
                 (_ (funcall reject event))))))
          (unless dice-roll-build-silent (pop-to-buffer buffer))
          nil)
      (funcall reject "dice-roll not build"))))

(defun dice-roll--ensure-native () "Ensure, that all native components are compiled. Return promise."
       (unless dice-roll--ensure-native-promise
         (setq dice-roll--ensure-native-promise
               (promise-new #'dice-roll--setup-function)))

       dice-roll--ensure-native-promise)


(provide 'dice-roll)
;;; dice-roll.el ends here
